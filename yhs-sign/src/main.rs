use alpha_sign::AlphaSign;
use clap::Parser;
use serialport::SerialPort;
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use yhs_sign::sign::talk_to_sign;
use yhs_sign::{api::app, AppState};

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
        .timeout(Duration::from_millis(1000))
        .parity(serialport::Parity::None)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .open()
        .expect("Failed to open port");

    let yhs_sign = AlphaSign::default();
    // yhs_sign.checksum = false;

    let (sign_command_tx, sign_command_rx) = tokio::sync::mpsc::unbounded_channel();

    let cancel_sign = CancellationToken::new();
    let cancel_sign_task = cancel_sign.clone();

    let mut app_state = AppState::new(sign_command_tx);
    app_state
        .try_load(
            // Value relied on elsewhere, search for
            // fd3e6cfb-3a3b-4b66-8eb2-5d54d6c91215
            &PathBuf::from("/var/data/yhs-sign/yhs-sign"),
        )
        .await;

    let message_loop = talk_to_sign(
        app_state.clone(),
        sign_command_rx,
        yhs_sign,
        port,
        cancel_sign_task,
    );
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

/// Serve the API.
///
/// # Arguments
/// * `app_state`: State shared between requests and the main application.
/// * `port`: Port to serve on.
pub async fn serve_api(app_state: AppState, port: u16) {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    tracing::info!("Listening on {}", addr);
    let _ = axum::Server::bind(&addr)
        .serve(app(app_state).into_make_service())
        .await;
}
