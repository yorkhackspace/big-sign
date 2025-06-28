use std::collections::HashMap;

use clap::{command, Parser, Subcommand};
use serde::Deserialize;

/// Response to a GET to /topics
#[derive(Debug, Deserialize)]
struct GetTopicsResponse {
    /// Available topics
    #[allow(unused)]
    topics: HashMap<String, Vec<String>>,
}

/// CLI to interact with BIG sign.
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Command to execute.
    #[command(subcommand)]
    command: CLICommand,
}

/// Available commands.
#[derive(Debug, Subcommand)]
enum CLICommand {
    /// Get all of the topics on the sign.
    GetTopics,
}

fn main() {
    let args = Args::parse();

    match args.command {
        CLICommand::GetTopics => {
            let client = reqwest::blocking::Client::new();
            let response = client
                .get("http://big-sign.yhs:8080/topics")
                .send()
                .unwrap()
                .text()
                .unwrap();
            let response: GetTopicsResponse = serde_json::from_str(&response).unwrap();
            println!("{response:?}");
        }
    }
}
