use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

const MAX_FAILURES: u32 = 5;
const BLOCK_WINDOW_SECONDS: i64 = 15 * 60;

#[derive(Clone, Debug, Default)]
struct AttemptState {
    failure_count: u32,
    blocked_until: Option<i64>,
}

fn store() -> &'static Mutex<HashMap<String, AttemptState>> {
    static STORE: OnceLock<Mutex<HashMap<String, AttemptState>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn check_login_allowed(identifier: &str, ip: Option<&str>, now: i64) -> Option<i64> {
    let key = attempt_key(identifier, ip);
    let guard = store().lock().expect("login limiter poisoned");
    let state = guard.get(&key)?;
    let blocked_until = state.blocked_until?;
    (blocked_until > now).then_some(blocked_until)
}

pub fn record_login_failure(identifier: &str, ip: Option<&str>, now: i64) -> i64 {
    let key = attempt_key(identifier, ip);
    let mut guard = store().lock().expect("login limiter poisoned");
    let state = guard.entry(key).or_default();
    state.failure_count += 1;
    if state.failure_count >= MAX_FAILURES {
        let blocked_until = now + BLOCK_WINDOW_SECONDS;
        state.blocked_until = Some(blocked_until);
        state.failure_count = 0;
        blocked_until
    } else {
        0
    }
}

pub fn clear_login_failures(identifier: &str, ip: Option<&str>) {
    let key = attempt_key(identifier, ip);
    let mut guard = store().lock().expect("login limiter poisoned");
    guard.remove(&key);
}

fn attempt_key(identifier: &str, ip: Option<&str>) -> String {
    format!(
        "{}:{}",
        identifier.trim().to_ascii_lowercase(),
        ip.unwrap_or("unknown")
    )
}
