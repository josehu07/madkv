//! Randomization related helpers.

use std::mem;

use rand::distr::{Alphanumeric, SampleString};
use rand::{rng, Rng};

use bit_vec::BitVec;

use super::{Stats, VALUE_LEN};
use runner::KvCall;

/// Generate a random index in the range [0, max).
pub(crate) fn gen_rand_index(max: usize) -> usize {
    debug_assert_ne!(max, 0);
    rng().random_range(0..max)
}

/// Generate a random alphanumeric string.
pub(crate) fn gen_rand_string(len: usize) -> String {
    debug_assert_ne!(len, 0);
    Alphanumeric.sample_string(&mut rng(), len)
}

/// Generate a random decision on call vs. wait resp.
pub(crate) fn gen_call_vs_wait(ops_called: usize, num_ops: usize, flying: &BitVec) -> bool {
    if ops_called == num_ops || flying.all() {
        false
    } else if flying.none() {
        true
    } else {
        rng().random()
    }
}

/// Generate a random client index, retry until one with expected on-the-fly
/// status is found.
pub(crate) fn gen_rand_client(flying: &BitVec, want_flying: bool) -> usize {
    debug_assert!(want_flying || !flying.all());
    debug_assert!(!want_flying || !flying.none());
    loop {
        let cidx = gen_rand_index(flying.len());
        if flying.get(cidx).unwrap() == want_flying {
            return cidx;
        }
    }
}

/// Generate a random `KvCall` operation, updating statistics accordingly.
pub(crate) fn gen_rand_kvcall(keys: &[String], stats: &mut Stats, cidx: usize) -> KvCall {
    match gen_rand_index(10) {
        0..=1 => {
            let kidx = gen_rand_index(keys.len());
            stats.cnt_put += 1;
            stats.keys_freq[cidx][kidx] += 1;

            KvCall::Put {
                key: keys[kidx].clone(),
                value: gen_rand_string(VALUE_LEN),
            }
        }

        2..=3 => {
            let kidx = gen_rand_index(keys.len());
            stats.cnt_swap += 1;
            stats.keys_freq[cidx][kidx] += 1;

            KvCall::Swap {
                key: keys[kidx].clone(),
                value: gen_rand_string(VALUE_LEN),
            }
        }

        4..=6 => {
            let kidx = gen_rand_index(keys.len());
            stats.cnt_get += 1;
            stats.keys_freq[cidx][kidx] += 1;

            KvCall::Get {
                key: keys[kidx].clone(),
            }
        }

        7..=8 => {
            let ksidx = gen_rand_index(keys.len());
            let keidx = gen_rand_index(keys.len());
            let mut key_start = keys[ksidx].clone();
            let mut key_end = keys[keidx].clone();
            if key_end < key_start {
                mem::swap(&mut key_start, &mut key_end);
            }
            stats.cnt_scan += 1;
            stats.keys_freq[cidx][ksidx] += 1;
            stats.keys_freq[cidx][keidx] += 1;

            KvCall::Scan { key_start, key_end }
        }

        9 => {
            let kidx = gen_rand_index(keys.len());
            stats.cnt_delete += 1;
            stats.keys_freq[cidx][kidx] += 1;

            KvCall::Delete {
                key: keys[kidx].clone(),
            }
        }

        _ => panic!("random KvCall variant out of range"),
    }
}
