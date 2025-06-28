use std::{
    collections::VecDeque,
    fs,
    time::{Duration, Instant},
};

use alpha_sign::{text::WriteText, AlphaSign};
use serialport::SerialPort;
use tokio_util::sync::CancellationToken;

use crate::{api::APIEvent, AppState, TopicId};

/// The current state of the sign.
pub struct SignState {
    /// Current topic and its index into the topics vec.
    current_topic: (TopicId, VecDeque<String>),
    message_last_shown_at: Instant,
}

impl SignState {
    fn should_draw(&self) -> bool {
        self.message_last_shown_at.elapsed() >= Duration::from_secs(15)
    }

    fn notify_draw(&mut self) {
        self.message_last_shown_at = Instant::now();
    }

    async fn set_current_topic(&mut self, app_state: &mut AppState, topic_id: TopicId) {
        let Some(lines) = app_state.get_topic(&topic_id).await else {
            // TODO: Should probably do something about this.
            return;
        };

        self.current_topic = (topic_id, lines.into());
        // Long enough ago to trigger re-draw.
        self.message_last_shown_at = Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
    }
}

/// Enters a loop of communicating with the sign and handling commands sent into the message channel.
///
/// # Arguments
/// * `app_state`: Shared app state.
/// * `message_rx`: Receiver for events to be handled.
/// * `sign`: Information about the sign we're talking to.
/// * `port`: The serial port we are talking to the sign on.
/// * `cancel`: [`CancellationToken`] that can be used to stop the task from running.
pub async fn talk_to_sign(
    mut app_state: AppState,
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<APIEvent>,
    mut sign: AlphaSign,
    mut port: Box<dyn SerialPort>,
    cancel: CancellationToken,
) {
    let (first_topic_id, first_topic) = app_state.get_next_topic(None).await;
    let mut sign_state = SignState {
        current_topic: (first_topic_id, first_topic.into()),
        message_last_shown_at: Instant::now().checked_sub(Duration::from_secs(60)).unwrap(),
    };

    while !cancel.is_cancelled() {
        if sign_state.should_draw() {
            if let Some(next_line) = sign_state.current_topic.1.pop_front() {
                write_to_sign(&mut sign, &mut port, &next_line).await;
                sign_state.notify_draw();
            }

            if sign_state.current_topic.1.is_empty() {
                let (next_topic_id, next_topic) = app_state
                    .get_next_topic(Some(&sign_state.current_topic.0))
                    .await;
                sign_state.current_topic = (next_topic_id, next_topic.into());
            }
        }

        if let Ok(event) = message_rx.try_recv() {
            handle_event(&mut app_state, &mut sign_state, event).await;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Handle an [`APIEvent`]
///
/// # Arguments
/// * `app_state`: Shared app state.
/// * `sign_state`: State of writing to the sign.
/// * `event`: Event to handle.
async fn handle_event(app_state: &mut AppState, sign_state: &mut SignState, event: APIEvent) {
    match event {
        APIEvent::TopicsUpdated => {
            let topics = app_state.get_all_topics().await;
            fs::write(
                // Value relied on elsewhere, search for
                // fd3e6cfb-3a3b-4b66-8eb2-5d54d6c91215
                "/var/data/yhs-sign/yhs-sign",
                serde_json::to_string_pretty(&topics).expect("Must be serializable"),
            )
            .expect("Could not save topics");
        }
        APIEvent::JumpToTopic(topic_id) => {
            sign_state.set_current_topic(app_state, topic_id).await;
        }
        APIEvent::TopicDeleted(topic_id) => {
            if topic_id == sign_state.current_topic.0 {
                let (next_topic_id, next_topic) = app_state.get_next_topic(None).await;
                sign_state.current_topic = (next_topic_id, next_topic.into());
                sign_state.message_last_shown_at =
                    Instant::now().checked_sub(Duration::from_secs(60)).unwrap();
            }
        }
    }
}

/// Write a single line of text to the sign.
///
/// # Arguments
/// * `sign`: Information about the sign we're talking to.
/// * `port`: The serial port we are talking to the sign on.
/// * `text`: The text to write to the sign.
async fn write_to_sign(sign: &mut AlphaSign, port: &mut Box<dyn SerialPort>, text: &str) {
    let write_text_command = sign
        .encode(alpha_sign::Command::WriteText(WriteText::new(
            '0',
            text.to_string(),
        )))
        .unwrap();

    let _ = port.write(write_text_command.as_slice()).ok(); // TODO handle errors
    println!("{:X?}", write_text_command);
}
