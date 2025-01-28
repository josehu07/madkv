//! Reference client that demonstrates the stdin/out workload interface.

use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, BufRead, Write};
use std::str::SplitWhitespace;

/// KV operation call type.
enum KvCall {
    Put { key: String, value: String },
    Swap { key: String, value: String },
    Get { key: String },
    Scan { key_start: String, key_end: String },
    Delete { key: String },
    Stop,
}

/// KV operation response type.
enum KvResp {
    Put {
        key: String,
        found: bool,
    },
    Swap {
        key: String,
        old_value: Option<String>,
    },
    Get {
        key: String,
        value: Option<String>,
    },
    Scan {
        key_start: String,
        key_end: String,
        entries: Vec<(String, String)>,
    },
    Delete {
        key: String,
        found: bool,
    },
}

/// Get the next segment from an input line iterator.
fn expect_next_seg(segs: &mut SplitWhitespace) -> Result<String, io::Error> {
    segs.next().map(|s| s.into()).ok_or(io::Error::new(
        io::ErrorKind::InvalidInput,
        "invalid input line",
    ))
}

/// Parse an input line into a KV operation call.
fn parse_input_call(line: &str) -> Result<KvCall, io::Error> {
    let mut segs = line.split_whitespace();
    match segs.next() {
        Some("PUT") => Ok(KvCall::Put {
            key: expect_next_seg(&mut segs)?,
            value: expect_next_seg(&mut segs)?,
        }),
        Some("SWAP") => Ok(KvCall::Swap {
            key: expect_next_seg(&mut segs)?,
            value: expect_next_seg(&mut segs)?,
        }),
        Some("GET") => Ok(KvCall::Get {
            key: expect_next_seg(&mut segs)?,
        }),
        Some("SCAN") => Ok(KvCall::Scan {
            key_start: expect_next_seg(&mut segs)?,
            key_end: expect_next_seg(&mut segs)?,
        }),
        Some("DELETE") => Ok(KvCall::Delete {
            key: expect_next_seg(&mut segs)?,
        }),
        Some("STOP") => Ok(KvCall::Stop),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid input line",
        )),
    }
}

/// Handle a call (dummy logic).
fn handle_kv_call(call: KvCall, state: &mut BTreeMap<String, String>) -> Option<KvResp> {
    match call {
        KvCall::Put { key, value } => {
            let found = state.contains_key(&key);
            state.insert(key.clone(), value);
            Some(KvResp::Put { key, found })
        }
        KvCall::Swap { key, value } => {
            let old_value = state.insert(key.clone(), value);
            Some(KvResp::Swap { key, old_value })
        }
        KvCall::Get { key } => {
            let value = state.get(&key).cloned();
            Some(KvResp::Get { key, value })
        }
        KvCall::Scan { key_start, key_end } => {
            let entries = state
                .range(key_start.clone()..key_end.clone())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Some(KvResp::Scan {
                key_start,
                key_end,
                entries,
            })
        }
        KvCall::Delete { key } => {
            let found = state.remove(&key).is_some();
            Some(KvResp::Delete { key, found })
        }
        KvCall::Stop => None,
    }
}

/// Produce an output KV response line, write to stdout directly.
fn write_response(resp: KvResp, stdout: &mut io::StdoutLock) -> Result<(), io::Error> {
    match resp {
        KvResp::Put { key, found } => writeln!(
            stdout,
            "PUT {} {}",
            key,
            if found { "found" } else { "not_found" }
        ),
        KvResp::Swap { key, old_value } => writeln!(
            stdout,
            "SWAP {} {}",
            key,
            if let Some(old_value) = old_value {
                old_value
            } else {
                "null".into()
            }
        ),
        KvResp::Get { key, value } => writeln!(
            stdout,
            "GET {} {}",
            key,
            if let Some(value) = value {
                value
            } else {
                "null".into()
            }
        ),
        KvResp::Scan {
            key_start,
            key_end,
            entries,
        } => {
            writeln!(stdout, "SCAN {} {} BEGIN", key_start, key_end)?;
            for (k, v) in entries {
                writeln!(stdout, "  {} {}", k, v)?;
            }
            writeln!(stdout, "SCAN END")
        }
        KvResp::Delete { key, found } => writeln!(
            stdout,
            "DELETE {} {}",
            key,
            if found { "found" } else { "not_found" }
        ),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin_handle = io::stdin().lock();
    let mut stdout_handle = io::stdout().lock();
    let mut buffer = String::new();

    // a fake, non-server-side sorted map
    let mut state = BTreeMap::new();

    loop {
        buffer.clear();
        stdin_handle.read_line(&mut buffer)?;

        let call = parse_input_call(buffer.trim())?;
        let resp = handle_kv_call(call, &mut state);

        if let Some(resp) = resp {
            write_response(resp, &mut stdout_handle)?;
        } else {
            break;
        }
    }

    Ok(())
}
