//! Standard input/output workload interface.

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
