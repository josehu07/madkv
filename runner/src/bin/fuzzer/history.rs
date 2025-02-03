//! Approximate real-time causal consistency checker.

use std::collections::{HashMap, VecDeque};

use runner::KvResp;

/// Per-key, non-read-only operation record with timestamp span.
#[derive(Debug, Clone)]
struct UpdateSpan {
    ts_call: u64,
    ts_resp: u64,
    value: Option<String>,
}

/// Check queue entry with timestamp span.
#[derive(Debug, Clone)]
struct QueuedSpan {
    ts_call: u64,
    ts_resp: u64,
    resp: KvResp,
}

/// Trimmed history of per-client acknowledged operations.
#[derive(Debug)]
pub(crate) struct History {
    /// Queue of pending responses to check (naturally ordered by response
    /// timestamp).
    queue: VecDeque<QueuedSpan>,

    /// Per-key per-client trimmed history of acknowledged update operations.
    spans: HashMap<String, Vec<VecDeque<UpdateSpan>>>,

    /// Per-client max update resp timestamp seen.
    maxtr: Vec<u64>,
}

impl History {
    /// Create a new empty history for given number of clients and keys pool.
    pub(crate) fn new(num_clis: usize, keys: &[Vec<String>]) -> Self {
        let mut spans = HashMap::new();
        for cli_keys in keys {
            for key in cli_keys {
                if !spans.contains_key(key) {
                    spans.insert(
                        key.clone(),
                        vec![
                            VecDeque::<UpdateSpan>::from([UpdateSpan {
                                ts_call: 0,
                                ts_resp: 0,
                                value: None, // dummy Delete to simplify logic
                            }]);
                            num_clis
                        ],
                    );
                }
            }
        }

        History {
            queue: VecDeque::new(),
            spans,
            maxtr: vec![0; num_clis],
        }
    }

    /// Add a newly acknowledged response result to the check queue.
    pub(crate) fn add_to_queue(&mut self, ts_call: u64, ts_resp: u64, resp: KvResp) {
        debug_assert!(ts_call < ts_resp);
        debug_assert!(self.queue.is_empty() || self.queue.back().unwrap().ts_resp < ts_resp);

        self.queue.push_back(QueuedSpan {
            ts_call,
            ts_resp,
            resp,
        });
    }

    /// Get the number of remaining checks in the check queue.
    pub(crate) fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Add a newly acknowledged update to the history, possibly trimming the
    /// heads of the history and possibly triggering some pending results to
    /// get checked. Returns:
    ///   - `Some(Some(resp))` if the check of a `resp` failed
    ///   - `Some(None)` if update key is unexpected
    ///   - `None` if everything is still alright
    pub(crate) fn apply_update(
        &mut self,
        cidx: usize,
        ts_call: u64,
        ts_resp: u64,
        key: String,
        value: Option<String>,
    ) -> Option<Option<KvResp>> {
        if let Some(key_spans) = self.spans.get_mut(&key) {
            debug_assert!(cidx < self.maxtr.len());
            debug_assert!(cidx < key_spans.len());
            debug_assert!(
                key_spans[cidx].is_empty() || key_spans[cidx].back().unwrap().ts_resp < ts_call
            );

            key_spans[cidx].push_back(UpdateSpan {
                ts_call,
                ts_resp,
                value,
            });
            self.maxtr[cidx] = ts_resp;
            let min_coming_ts = *self.maxtr.iter().min().unwrap();
            let min_queued_ts = self
                .queue
                .iter()
                .map(|e| e.ts_call)
                .min()
                .unwrap_or(u64::MAX);

            // trim off updates at head of client histories
            for cli_spans in key_spans.iter_mut() {
                let mut keep_ts = 0;
                for span in cli_spans.iter().rev() {
                    // can discard only if there's one span that's fully ahead
                    // of the next possible incoming request and any pending
                    // request in the check queue
                    if span.ts_resp < min_coming_ts && span.ts_resp < min_queued_ts {
                        keep_ts = span.ts_call;
                        break;
                    }
                }
                while cli_spans.len() > 1 && cli_spans.front().unwrap().ts_resp < keep_ts {
                    cli_spans.pop_front();
                }
            }

            // pop off now-checkable results from the check queue
            while self.queue.front().is_some()
                && self.queue.front().unwrap().ts_resp < min_coming_ts
            {
                let entry = self.queue.pop_front().unwrap();
                if !self.check_call(&entry) {
                    return Some(Some(entry.resp));
                }
            }

            None
        } else {
            Some(None)
        }
    }

    /// Check a call popped off from the check queue, which is now decidable.
    fn check_call(&self, entry: &QueuedSpan) -> bool {
        match &entry.resp {
            KvResp::Put { key, found } => {
                if let Some(key_spans) = self.spans.get(key) {
                    Self::check_put(key_spans, entry.ts_call, entry.ts_resp, found)
                } else {
                    false
                }
            }
            KvResp::Swap { key, old_value } => {
                if let Some(key_spans) = self.spans.get(key) {
                    Self::check_swap(key_spans, entry.ts_call, entry.ts_resp, old_value.as_ref())
                } else {
                    false
                }
            }
            KvResp::Get { key, value } => {
                if let Some(key_spans) = self.spans.get(key) {
                    Self::check_get(key_spans, entry.ts_call, entry.ts_resp, value.as_ref())
                } else {
                    false
                }
            }
            KvResp::Scan {
                key_start,
                key_end,
                entries,
            } => Self::check_scan(
                &self.spans,
                entry.ts_call,
                entry.ts_resp,
                key_start,
                key_end,
                entries,
            ),
            KvResp::Delete { key, found } => {
                if let Some(key_spans) = self.spans.get(key) {
                    Self::check_delete(key_spans, entry.ts_call, entry.ts_resp, found)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check a Put operation result assuming given history.
    fn check_put(
        key_spans: &[VecDeque<UpdateSpan>],
        ts_call: u64,
        ts_resp: u64,
        found: &bool,
    ) -> bool {
        // println!(
        //     "--- PUT <{} - {}> {} {:?}",
        //     ts_call, ts_resp, found, key_spans
        // );
        for cli_spans in key_spans {
            for span in cli_spans.iter().rev() {
                if span.ts_call < ts_resp && span.value.is_some() == *found {
                    return true;
                }
                if span.ts_resp < ts_call {
                    break;
                }
            }
        }
        false
    }

    /// Check a Swap operation result assuming given history.
    fn check_swap(
        key_spans: &[VecDeque<UpdateSpan>],
        ts_call: u64,
        ts_resp: u64,
        old_value: Option<&String>,
    ) -> bool {
        // println!(
        //     "--- SWAP <{} - {}> {:?} {:?}",
        //     ts_call, ts_resp, old_value, key_spans
        // );
        for cli_spans in key_spans {
            for span in cli_spans.iter().rev() {
                if span.ts_call < ts_resp && span.value.as_ref() == old_value {
                    return true;
                }
                if span.ts_resp < ts_call {
                    break;
                }
            }
        }
        false
    }

    /// Check a Get operation result assuming given history.
    fn check_get(
        key_spans: &[VecDeque<UpdateSpan>],
        ts_call: u64,
        ts_resp: u64,
        value: Option<&String>,
    ) -> bool {
        // println!(
        //     "--- GET <{} - {}> {:?} {:?}",
        //     ts_call, ts_resp, value, key_spans
        // );
        for cli_spans in key_spans {
            for span in cli_spans.iter().rev() {
                if span.ts_call < ts_resp && span.value.as_ref() == value {
                    return true;
                }
                if span.ts_resp < ts_call {
                    break;
                }
            }
        }
        false
    }

    /// Check a Scan operation result assuming given history. All possible
    /// keys in range are searched here.
    fn check_scan(
        spans: &HashMap<String, Vec<VecDeque<UpdateSpan>>>,
        ts_call: u64,
        ts_resp: u64,
        key_start: &String,
        key_end: &String,
        entries: &[(String, String)],
    ) -> bool {
        let mut entries_map = HashMap::new();
        for (key, value) in entries {
            if key < key_start || key > key_end {
                return false; // out-of-range in scan result
            }
            if entries_map.contains_key(key) {
                return false; // duplicate key in scan result
            }
            entries_map.insert(key, value);
        }

        // println!("--- SCAN <{} - {}> loop", ts_call, ts_resp);
        for (key, key_spans) in spans {
            // if key >= key_start && key <= key_end {
            //     println!("... {} {:?} {:?}", key, entries_map.get(key), key_spans);
            // }
            if key >= key_start
                && key <= key_end
                && !Self::check_get(key_spans, ts_call, ts_resp, entries_map.get(key).copied())
            {
                return false;
            }
        }
        true // all possible keys in range passed check
    }

    /// Check a Delete operation result assuming given history.
    fn check_delete(
        key_spans: &[VecDeque<UpdateSpan>],
        ts_call: u64,
        ts_resp: u64,
        found: &bool,
    ) -> bool {
        // println!(
        //     "--- DELETE <{} - {}> {} {:?}",
        //     ts_call, ts_resp, found, key_spans
        // );
        for cli_spans in key_spans {
            for span in cli_spans.iter().rev() {
                if span.ts_call < ts_resp && span.value.is_some() == *found {
                    return true;
                }
                if span.ts_resp < ts_call {
                    break;
                }
            }
        }
        false
    }
}
