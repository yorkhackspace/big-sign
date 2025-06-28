use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{fs, sync::Mutex};

use crate::api::APIEvent;

pub mod api;
pub mod sign;

const PLACEHOLDER_TOPIC_ID: &str = "__PLACEHOLDER";
const PLACEHOLDER_TOPIC_TEXT: &str = "Welcome to York Hackspace";
const TUTORIAL_TOPIC_ID: &str = "__TUTORIAL";
const TUTORIAL_TOPIC_TEXT: &str = "http://big-sign.yhs:8080/help";

/// State shared between the main application and the HTTP application.
#[derive(Clone)]
pub struct AppState {
    /// Message channel into which events can be sent.
    event_tx: tokio::sync::mpsc::UnboundedSender<APIEvent>,

    inner_state: Arc<Mutex<AppStateInner>>,
}

#[derive(Debug, Default)]
pub struct AppStateInner {
    messages: HashMap<TopicId, Vec<String>>,
    topic_ids: Vec<TopicId>,
}

impl AppState {
    /// Creates a new [`AppState`].
    ///
    /// # Arguments
    /// * `event_tx`: Channel into which events can be sent.
    ///
    /// # Returns
    /// A new [`AppState`].
    pub fn new(event_tx: tokio::sync::mpsc::UnboundedSender<APIEvent>) -> Self {
        Self {
            event_tx,
            inner_state: Default::default(),
        }
    }

    pub async fn try_load(&mut self, path: &PathBuf) {
        // TODO: Errors.
        if let Ok(data) = fs::read_to_string(path).await {
            let data_decoded: serde_json::Result<HashMap<TopicId, Vec<String>>> =
                serde_json::from_str(&data);
            if let Ok(data) = data_decoded {
                for (topic_id, lines) in data {
                    self.set_topic(&topic_id, lines).await;
                }
            }
        }

        self.set_topic(
            &TopicId(TUTORIAL_TOPIC_ID.to_string()),
            [TUTORIAL_TOPIC_TEXT.to_string()].to_vec(),
        )
        .await;
    }
    pub async fn set_topic(&mut self, topic_id: &TopicId, lines: Vec<String>) {
        if lines.iter().any(|line| line.len() > 60) {
            return; // TODO: Error.
        }

        let mut state_lock = self.inner_state.lock().await;
        state_lock.messages.insert(topic_id.clone(), lines);
        if !state_lock.topic_ids.contains(topic_id) {
            state_lock.topic_ids.push(topic_id.clone());
        }
    }

    pub async fn delete_topic(&mut self, topic_id: &TopicId) {
        let mut state_lock = self.inner_state.lock().await;
        state_lock.messages.remove(topic_id);
        if let Some(index) = state_lock
            .topic_ids
            .iter()
            .position(|value| value == topic_id)
        {
            state_lock.topic_ids.remove(index);
        }
    }

    pub async fn get_all_topics(&mut self) -> HashMap<TopicId, Vec<String>> {
        let state_lock = self.inner_state.lock().await;
        state_lock.messages.clone()
    }

    pub async fn get_topic(&self, topic_id: &TopicId) -> Option<Vec<String>> {
        let state_lock = self.inner_state.lock().await;
        state_lock.messages.get(topic_id).cloned()
    }

    pub async fn get_next_topic(&self, topic_id: Option<&TopicId>) -> (TopicId, Vec<String>) {
        let state_lock = self.inner_state.lock().await;
        let topic_index = match topic_id {
            Some(topic_id) => state_lock
                .topic_ids
                .iter()
                .position(|value| value == topic_id),
            None => None,
        };

        let next_topic = match topic_index {
            Some(mut index) => {
                index += 1;
                if index >= state_lock.topic_ids.len() {
                    index = 0;
                }

                index
            }
            None => 0,
        };

        match state_lock.topic_ids.get(next_topic) {
            Some(topic_id) => {
                let lines = state_lock
                    .messages
                    .get(topic_id)
                    .expect("Topics Vec out of sync with HashMap");
                (topic_id.clone(), lines.clone())
            }
            None => (
                TopicId(PLACEHOLDER_TOPIC_ID.to_string()),
                vec![PLACEHOLDER_TOPIC_TEXT.to_string()],
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicId(String);
