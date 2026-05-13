use std::io::{self, BufRead, BufReader, Read};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use yoyopod_protocol::{ProtocolError, WorkerEnvelope};

pub(crate) fn spawn_line_reader<R>(input: R) -> Receiver<io::Result<String>>
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let reader = BufReader::new(input);
        for line in reader.lines() {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
}

pub(crate) fn decode_envelope(line: &str) -> Result<WorkerEnvelope, ProtocolError> {
    WorkerEnvelope::decode(line.as_bytes())
}

pub(crate) fn encode_envelope(envelope: &WorkerEnvelope) -> Result<Vec<u8>, ProtocolError> {
    envelope.encode()
}
