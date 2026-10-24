//! Explicit system print-queue adapter. Preview never submits a job.
use crate::Result;
use std::{path::Path, process::Command};
pub fn supported() -> bool {
    cfg!(any(target_os = "macos", target_os = "linux"))
}
pub fn printers() -> Result<String> {
    if !supported() {
        return Err("Native print queue discovery is unavailable on this platform.".into());
    }
    let out = Command::new("lpstat")
        .args(["-p", "-d"])
        .output()
        .map_err(|e| format!("Cannot query printers: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into())
}
pub fn submit(path: &Path, queue: &str) -> Result<String> {
    if !supported() {
        return Err("Export PDF and use system printing on this platform.".into());
    }
    if queue.is_empty()
        || queue.len() > 127
        || !queue
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
    {
        return Err("Use the exact queue name (letters, digits, underscore or hyphen).".into());
    }
    let out = Command::new("lp")
        .arg("-d")
        .arg(queue)
        .arg("--")
        .arg(path)
        .output()
        .map_err(|e| format!("Cannot submit print job: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "Print submission failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(format!(
        "Submitted to system queue: {}. This does not confirm physical printing.",
        String::from_utf8_lossy(&out.stdout).trim()
    ))
}
