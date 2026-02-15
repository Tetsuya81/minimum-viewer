// #region agent log
use std::collections::BTreeMap;
use std::io::Write;

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[allow(dead_code)]
pub fn log(location: &str, message: &str, data: BTreeMap<&str, String>, hypothesis_id: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".cursor/debug.log");
    let ts: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let data_str = data
        .iter()
        .map(|(k, v)| format!("{}:{}", quote(k), quote(v)))
        .collect::<Vec<_>>()
        .join(",");
    let line = format!(
        "{{\"location\":\"{}\",\"message\":\"{}\",\"data\":{{{}}},\"timestamp\":{},\"hypothesisId\":\"{}\"}}\n",
        quote(location),
        quote(message),
        data_str,
        ts,
        hypothesis_id
    );
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| {
            std::io::Write::write_all(&mut f, line.as_bytes())?;
            f.flush()
        });
}
// #endregion
