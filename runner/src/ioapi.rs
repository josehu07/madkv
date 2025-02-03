//! Standard input/output workload interface.

use std::io;
use std::str::SplitWhitespace;

use strum::EnumCount;

use crate::RunnerError;

/// KV operation call type.
#[derive(Debug, Clone, EnumCount)]
pub enum KvCall {
    Put { key: String, value: String },
    Swap { key: String, value: String },
    Get { key: String },
    Scan { key_start: String, key_end: String },
    Delete { key: String },
    Stop,
}

impl KvCall {
    /// Write a KV operation call as a string line to a writer.
    pub(crate) fn into_write(self, writer: &mut impl io::Write) -> Result<(), RunnerError> {
        match self {
            KvCall::Put { key, value } => Ok(writeln!(writer, "PUT {} {}", key, value)?),
            KvCall::Swap { key, value } => Ok(writeln!(writer, "SWAP {} {}", key, value)?),
            KvCall::Get { key } => Ok(writeln!(writer, "GET {}", key)?),
            KvCall::Scan { key_start, key_end } => {
                Ok(writeln!(writer, "SCAN {} {}", key_start, key_end)?)
            }
            KvCall::Delete { key } => Ok(writeln!(writer, "DELETE {}", key)?),
            KvCall::Stop => Ok(writeln!(writer, "STOP")?),
        }
    }

    /// Returns the value update made by this operation call:
    ///   - `None` if read-only operation
    ///   - `Some((key, None))` if Delete operation
    ///   - `Some((key, Some(value)))` if Put or Swap operation
    pub fn update_info(&self) -> Option<(String, Option<String>)> {
        match self {
            KvCall::Put { key, value, .. } => Some((key.clone(), Some(value.clone()))),
            KvCall::Swap { key, value, .. } => Some((key.clone(), Some(value.clone()))),
            KvCall::Delete { key, .. } => Some((key.clone(), None)),
            _ => None,
        }
    }
}

/// KV operation response type.
#[derive(Debug, Clone, EnumCount)]
pub enum KvResp {
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
    Stop,
}

impl KvResp {
    /// Read the next line from reader into a thread-local buffer.
    fn read_next_line(
        mut reader: impl io::BufRead,
        buffer: &mut String,
    ) -> Result<usize, RunnerError> {
        buffer.clear();
        let size = loop {
            let size = reader.read_line(buffer)?;
            // skip empty lines
            if !buffer.trim().is_empty() {
                break size;
            }
        };
        Ok(size)
    }

    /// Return an iterator over a line's whitespace-delimited segments.
    fn get_segs_of_line(buffer: &str) -> SplitWhitespace {
        buffer.split_whitespace()
    }

    /// Expect the next segment from an iterator of segments, returning an
    /// error if there's no next segment.
    fn expect_next_seg(segs: &mut SplitWhitespace, buffer: &str) -> Result<String, RunnerError> {
        segs.next()
            .map(|s| s.into())
            .ok_or(RunnerError::Io(format!("invalid line: {}", buffer)))
    }

    /// Construct a KV operation response from a reader.
    pub(crate) fn from_read(
        reader: &mut impl io::BufRead,
        buffer: &mut String,
    ) -> Result<KvResp, RunnerError> {
        Self::read_next_line(&mut *reader, buffer)?;
        let mut segs = Self::get_segs_of_line(buffer);

        match segs.next() {
            Some("PUT") => Ok(KvResp::Put {
                key: Self::expect_next_seg(&mut segs, buffer)?,
                found: {
                    let found = Self::expect_next_seg(&mut segs, buffer)?;
                    if found == "found" {
                        true
                    } else if found == "not_found" {
                        false
                    } else {
                        return Err(RunnerError::Parse(format!(
                            "invalid 'found' field: {}",
                            found
                        )));
                    }
                },
            }),

            Some("SWAP") => Ok(KvResp::Swap {
                key: Self::expect_next_seg(&mut segs, buffer)?,
                old_value: {
                    let old_value = Self::expect_next_seg(&mut segs, buffer)?;
                    if old_value == "null" {
                        None
                    } else {
                        Some(old_value)
                    }
                },
            }),

            Some("GET") => Ok(KvResp::Get {
                key: Self::expect_next_seg(&mut segs, buffer)?,
                value: {
                    let value = Self::expect_next_seg(&mut segs, buffer)?;
                    if value == "null" {
                        None
                    } else {
                        Some(value)
                    }
                },
            }),

            Some("SCAN") => {
                let key_start = Self::expect_next_seg(&mut segs, buffer)?;
                let key_end = Self::expect_next_seg(&mut segs, buffer)?;
                let begin = Self::expect_next_seg(&mut segs, buffer)?;
                if begin != "BEGIN" {
                    return Err(RunnerError::Io(format!("invalid line: {}", buffer)));
                }

                // loop through scan results
                let mut entries = vec![];
                loop {
                    Self::read_next_line(&mut *reader, buffer)?;
                    if buffer.trim() == "SCAN END" {
                        break;
                    }
                    let mut entry_segs = Self::get_segs_of_line(buffer);

                    let key = Self::expect_next_seg(&mut entry_segs, buffer)?;
                    let value = Self::expect_next_seg(&mut entry_segs, buffer)?;
                    entries.push((key, value));
                }

                Ok(KvResp::Scan {
                    key_start,
                    key_end,
                    entries,
                })
            }

            Some("DELETE") => Ok(KvResp::Delete {
                key: Self::expect_next_seg(&mut segs, buffer)?,
                found: {
                    let found = Self::expect_next_seg(&mut segs, buffer)?;
                    if found == "found" {
                        true
                    } else if found == "not_found" {
                        false
                    } else {
                        return Err(RunnerError::Parse(format!(
                            "invalid 'found' field: {}",
                            found
                        )));
                    }
                },
            }),

            Some("STOP") => Ok(KvResp::Stop),

            _ => Err(RunnerError::Io(format!("invalid line: {}", buffer))),
        }
    }
}
