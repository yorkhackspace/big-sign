use clap::Parser;
use rhai::EvalAltResult;
use serialport::SerialPort;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    thread,
    time::Duration,
};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use yhs_sign::{
    commands::text::WriteText,
    web_server::{app, AppState},
    AlphaSign, SignCommand, SignSerial,
};

/// Service for communicating with the YHS sign.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Whether to print serial communication via the logger, rather than trying to talk to a serial port. Commands will be logged at the `trace` log level.
    #[arg(long)]
    fake_serial: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    dotenv::dotenv().ok();
    init_logging();

    tracing::info!("🦊 Hello YHS! 🦊");

    let port: Box<dyn SignSerial> = if args.fake_serial {
        Box::new(LoggerSerialPort)
    } else {
        let port = serialport::new("/dev/ttyUSB0", 9600)
            .timeout(Duration::from_millis(10))
            .parity(serialport::Parity::None)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .open()
            .expect("Failed to open port");
        Box::new(WrappedSerialPort(port))
    };

    let yhs_sign = AlphaSign::new(port, [0x30, 0x30], yhs_sign::TypeCode::AllSigns);

    let (sign_command_tx, sign_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let cancel_sign = CancellationToken::new();
    let cancel_sign_task = cancel_sign.clone();

    let app_state = AppState::new(sign_command_tx);

    let message_loop = talk_to_sign(yhs_sign, sign_command_rx, cancel_sign_task);
    let http_api = serve_api(app_state, 8080);

    select! {
        _ = message_loop => {},
        _ = http_api => {},
    }

    cancel_sign.cancel();
}

/// Wraps a [`Box<dyn SerialPort>`] so that it can be treated as a generic [`SignSerial`].
struct WrappedSerialPort(Box<dyn SerialPort>);

impl SignSerial for WrappedSerialPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.0.write(buf)
    }
}

/// An implementer of [`SignSerial`] that logs to the console at the `trace` log level.
#[derive(Default)]
struct LoggerSerialPort;

impl SignSerial for LoggerSerialPort {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        tracing::trace!("Serial data to sign: {:02X?}", buf);

        Ok(buf.len())
    }
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
    mut message_rx: tokio::sync::mpsc::UnboundedReceiver<SignCommand>,
    cancel: CancellationToken,
) {
    let sign = Arc::new(tokio::sync::Mutex::new(sign));
    let rhai_engine = make_rhai_engine(sign.clone());

    while !cancel.is_cancelled() {
        select! {
            _ = cancel.cancelled() => {},
            message = message_rx.recv() => {
                match message {
                    Some(command) => {
                        handle_command(&sign, &rhai_engine, command).await;
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

/// Makes a [`rhai::Engine`] that can be used to execute scripts written in the rhai language.
/// Registers handlers and custom functions
///
/// # Arguments
/// * `sign`: The sign to send commands to.
///
/// # Returns
/// A [`rhai::Engine`] with handlers for print, debug, and custom functions for sign shenanigans.
fn make_rhai_engine(sign: Arc<tokio::sync::Mutex<AlphaSign>>) -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.on_print(move |s| {
        tracing::info!("From user-provided script: {s}");
    });
    engine.on_debug(move |s, src, pos| {
        let src = src.unwrap_or("unknown");
        tracing::debug!("From user-provided script: {src} at {pos:?}: {s}");
    });
    engine.register_fn("write", move |text: &str| {
        let mut sign_lock = pollster::block_on(sign.lock());
        let write_text_command = WriteText::new('A', text.to_string());
        sign_lock.send_command(write_text_command);
    });
    engine.register_fn(
        "delay_seconds",
        move |seconds: i64| -> Result<(), Box<EvalAltResult>> {
            if seconds < 0 {
                return Err("Cannot wait for a negtive duration!".into());
            }

            if let Ok(seconds) = TryInto::<u64>::try_into(seconds) {
                thread::sleep(Duration::from_secs(seconds));
                Ok(())
            } else {
                Err("Invalid wait duration!".into())
            }
        },
    );

    engine
}

/// Handle a [`SignCommand`]
///
/// # Arguments
/// * `sign`: The sign to send commands to.
/// * `rhai_engine`: Engine to use for executing Rhai scripts.
/// * `command`: The command to handle.
async fn handle_command(
    sign: &tokio::sync::Mutex<AlphaSign>,
    rhai_engine: &rhai::Engine,
    command: SignCommand,
) {
    match command {
        SignCommand::WriteText { text } => {
            let mut sign_lock = sign.lock().await;
            let write_text_command = WriteText::new('A', text);
            sign_lock.send_command(write_text_command);
        }
        SignCommand::RunScript {
            script_language,
            script,
        } => match script_language {
            yhs_sign::SignScriptLanguage::Rhai => {
                if let Err(err) = rhai_engine.run(&script) {
                    tracing::error!("Error from user-provided script: {}", err);
                }
            }
        },
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
