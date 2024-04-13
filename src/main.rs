mod web_server;

use crate::web_server::{app, AppState};
use alpha_sign::AlphaSign;
use alpha_sign::Command;
use clap::Parser;
// use rhai::EvalAltResult;
use serialport::SerialPort;
use std::{
    net::{Ipv4Addr, SocketAddr},
    //    thread,
    time::Duration,
};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

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

    let port: Box<dyn SerialPort> = serialport::new(args.port.as_str(), args.baudrate)
        .timeout(Duration::from_millis(10))
        .parity(serialport::Parity::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .open()
        .expect("Failed to open port");

    let yhs_sign = AlphaSign::default();

    let (sign_command_tx, sign_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let cancel_sign = CancellationToken::new();
    let cancel_sign_task = cancel_sign.clone();

    let app_state = web_server::AppState::new(sign_command_tx);

    let message_loop = talk_to_sign(yhs_sign, port, sign_command_rx, cancel_sign_task);
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
    sign: AlphaSign,
    port: Box<dyn SerialPort>,
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<Command>,
    cancel: CancellationToken,
) {
    // let rhai_engine = make_rhai_engine(sign.clone());
    let port_lock = tokio::sync::Mutex::new(port);
    while !cancel.is_cancelled() {
        select! {
            _ = cancel.cancelled() => {},
            message = message_rx.recv() => {
                match message {
                    Some(command) => {
                        handle_command(&sign, &port_lock, command).await;
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

/// Handle a [`SignCommand`]
///
/// # Arguments
/// * `sign`: The sign to send commands to.
/// * `port`: the serial port to send things down
/// * `command`: The command to handle.
async fn handle_command(
    sign: &AlphaSign,
    port: &tokio::sync::Mutex<Box<dyn SerialPort>>,
    // rhai_engine: &rhai::Engine,
    command: Command,
) {
    match command {
        Command::WriteText(text) => {
            let mut port_lock = port.lock().await;
            let write_text_command = sign.encode(vec![Command::WriteText(text)]);
            port_lock.write(write_text_command.as_slice()).ok(); // TODO handle errors
        }
        Command::ReadText(_) => todo!(),
        Command::WriteSpecial(_) => todo!(),
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
