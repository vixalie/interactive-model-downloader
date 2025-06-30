use std::time::Duration;

pub fn duration_to_sec_string(duration: &Duration) -> String {
    let sec = duration.as_secs();
    format!("{sec}s")
}
