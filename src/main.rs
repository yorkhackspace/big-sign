mod web_server;

use crate::web_server::{app, AppState};
use alpha_sign::text::WriteText;
use alpha_sign::Command;
use alpha_sign::Packet;
use alpha_sign::SignSelector;
use clap::Parser;
// use rhai::EvalAltResult;
use serialport::SerialPort;
use std::io::BufRead;
use std::io::BufReader;
use std::{
    net::{Ipv4Addr, SocketAddr},
    //    thread,
    time::Duration,
};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use web_server::APICommand;

/// Service for communicating with the YHS sign.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // serial port to use to connect to the sign
    #[arg(long, default_value = "/dev/ttyUSB0")]
    port: String,
    // baud rate to use for the port
    #[arg(long, default_value = "9600")]
    baudrate: u32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    dotenv::dotenv().ok();
    init_logging();

    tracing::info!("🦊 Hello YHS! 🦊");

    let mut port: Box<dyn SerialPort> = serialport::new(args.port.as_str(), args.baudrate)
        .timeout(Duration::from_millis(1000))
        .parity(serialport::Parity::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .open()
        .expect("Failed to open port");

    let yhs_selector = SignSelector::default();
    // yhs_selector.checksum = false;

    let (sign_command_tx, sign_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let cancel_sign = CancellationToken::new();
    let cancel_sign_task = cancel_sign.clone();

    let app_state = web_server::AppState::new(sign_command_tx);

    let message_loop = talk_to_sign(yhs_selector, port, sign_command_rx, cancel_sign_task);
    let http_api = serve_api(app_state, 8080);

    select! {
        _ = message_loop => {},
        _ = http_api => {},
    }

    cancel_sign.cancel();
}

/// Set up logging.
fn init_logging() {
    #[cfg(debug_assertions)]
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "yhs_sign=info")
    }

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    let env_filter = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(stdout_log.with_filter(env_filter))
        .init();
}

/// Enters a loop of communicating with the sign and handling commands sent into the message channel.
///
/// # Arguments
/// * `sign`: The sign to talk to.
/// * `message_rx`: Receiver for commands to be handled.
/// * `cancel`: [`CancellationToken`] that can be used to stop the task from running.
async fn talk_to_sign(
    sign: SignSelector,
    mut port: Box<dyn SerialPort>,
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<APICommand>,
    cancel: CancellationToken,
) {
    while !cancel.is_cancelled() {
        select! {
            _ = cancel.cancelled() => {},
            message = message_rx.recv() => {
                match message {
                    Some(command) => {
                        handle_command(sign, &mut port, command).await;
                    }
                    None => {
                        tracing::debug!(
                            "Command channel was closed, exiting loop of communicating with sign"
                        );
                        cancel.cancel()
                    }
                }
            }
        }
    }
}

/// Handle a [`APICommand`]
///
/// # Arguments
/// * `sign`: The sign to send commands to.
/// * `port`: the serial port to send things down
/// * `command`: The command to handle.
async fn handle_command(sign: SignSelector, port: &mut Box<dyn SerialPort>, command: APICommand) {
    match command {
        APICommand::WriteText(text) => {
            let write_text_command =
                Packet::new(vec![sign], vec![Command::WriteText(text)]).encode();

            port.write(write_text_command.as_slice()).ok(); // TODO handle errors
        }
        APICommand::ReadText(command, tx) => {
            let read_text_command =
                Packet::new(vec![sign], vec![Command::ReadText(command)]).encode();

            port.write(read_text_command.as_slice()).ok();

            let mut bufreader = BufReader::new(port);

            let mut buf: Vec<u8> = vec![];

            bufreader.read_until(0x04, &mut buf).ok();

            let (_, parse) = Packet::parse(buf.as_slice()).expect("error parsing response"); // TODO error handling

            if let Command::WriteText(WriteText { message: t, .. }) = &parse.commands[0] {
                tx.send(web_server::APIResponse::ReadText(t.clone())).ok();
            }
        }
    }
}

/// Serve the API.
///
/// # Arguments
/// * `app_state`: State shared between requests and the main application.
/// * `port`: Port to serve on.
async fn serve_api(app_state: AppState, port: u16) {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    tracing::info!("Listening on {}", addr);
    let _ = axum::Server::bind(&addr)
        .serve(app(app_state).into_make_service())
        .await;
}
