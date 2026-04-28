use anyhow::Result;
use clap::Parser;
use serde_json::json;
use std::io::{self, BufRead, Write};

mod config;
mod events;
mod host;
mod protocol;

use protocol::WorkerEnvelope;

#[derive(Debug, Parser)]
#[command(name = "yoyopod-voip-host")]
#[command(about = "YoYoPod Rust VoIP host")]
struct Args {
    #[arg(long, default_value = "")]
    shim_path: String,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    write_envelope(&WorkerEnvelope::event(
        "voip.ready",
        json!({"capabilities":["calls"]}),
    ))?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let envelope = match WorkerEnvelope::decode(line.as_bytes()) {
            Ok(envelope) => envelope,
            Err(error) => {
                write_envelope(&WorkerEnvelope::error(
                    "voip.error",
                    None,
                    "protocol_error",
                    error.to_string(),
                ))?;
                continue;
            }
        };

        if envelope.message_type == "voip.health" {
            write_envelope(&WorkerEnvelope::result(
                "voip.health",
                envelope.request_id,
                json!({"ready":true,"registered":false,"active_call_id":null}),
            ))?;
        } else if envelope.message_type == "voip.shutdown" {
            break;
        } else {
            write_envelope(&WorkerEnvelope::error(
                "voip.error",
                envelope.request_id,
                "unsupported_command",
                format!("unsupported command {}", envelope.message_type),
            ))?;
        }
    }
    Ok(())
}

fn write_envelope(envelope: &WorkerEnvelope) -> Result<()> {
    let encoded = envelope.encode()?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&encoded)?;
    stdout.flush()?;
    Ok(())
}
