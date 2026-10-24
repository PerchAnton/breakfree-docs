//! Office process contract v1: no window or graphical context on information paths.
use crate::{Result, VERSION};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum Launch {
    New,
    Open(PathBuf),
    Capabilities,
    Version,
    Help,
}
pub fn arguments(args: impl IntoIterator<Item = OsString>) -> Result<Launch> {
    let mut args = args.into_iter();
    let first = args.next();
    if first.as_ref().is_some_and(|a| a.to_str().is_none()) {
        return Err("Invalid command encoding.".into());
    }
    let result = match first.as_deref().and_then(|a| a.to_str()) {
        None | Some("--new") => Launch::New,
        Some("--capabilities") => Launch::Capabilities,
        Some("--version") => Launch::Version,
        Some("--help" | "-h") => Launch::Help,
        Some("--open") => {
            let path = PathBuf::from(args.next().ok_or("--open requires an absolute path.")?);
            if !path.is_absolute() {
                return Err("--open requires an absolute path.".into());
            }
            Launch::Open(path)
        }
        _ => return Err("Unknown argument. Use --help.".into()),
    };
    if args.next().is_some() {
        return Err("Specify exactly one launch command.".into());
    }
    Ok(result)
}
pub fn capabilities() -> serde_json::Value {
    serde_json::json!({"protocol_version":1,"app_id":"breakfree.docs","app_version":VERSION,"display_name":"BreakFree Docs","capabilities":if cfg!(feature="desktop"){vec!["new","open"]}else{vec![]},"formats":[{"id":"breakfree.docs.native","extensions":["bfdoc"],"read":true,"write":true},{"id":"ooxml.wordprocessingml","extensions":["docx"],"read":true,"write":true}]})
}
