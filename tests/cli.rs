use breakfree_docs::{
    VERSION,
    integration::{self, Launch},
};
use std::{ffi::OsString, process::Command};
#[test]
fn validates_launch_arguments() {
    assert_eq!(
        integration::arguments([OsString::from("--new")]).unwrap(),
        Launch::New
    );
    let path = std::env::current_dir().unwrap().join("Example.bfdoc");
    assert_eq!(
        integration::arguments([OsString::from("--open"), path.clone().into_os_string()]).unwrap(),
        Launch::Open(path)
    );
    for args in [
        vec!["--open"],
        vec!["--open", "relative.bfdoc"],
        vec!["--new", "--version"],
        vec!["--unknown"],
        vec!["extra"],
    ] {
        assert!(integration::arguments(args.into_iter().map(OsString::from)).is_err());
    }
}
#[test]
fn capabilities_and_version_are_clean_outside_checkout() {
    let dir = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_breakfree-docs"))
        .arg("--capabilities")
        .current_dir(dir.path())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(result.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["protocol_version"], 1);
    assert_eq!(value["app_id"], "breakfree.docs");
    assert_eq!(value["app_version"], VERSION);
    assert_eq!(value["display_name"], "BreakFree Docs");
    assert_eq!(
        value["formats"][0]["extensions"],
        serde_json::json!(["bfdoc"])
    );
    assert_eq!(
        value["formats"],
        serde_json::json!([{"id":"breakfree.docs.native","extensions":["bfdoc"],"read":true,"write":true}])
    );
    assert_eq!(
        value["capabilities"],
        if cfg!(feature = "desktop") {
            serde_json::json!(["new", "open"])
        } else {
            serde_json::json!([])
        }
    );
    let version = Command::new(env!("CARGO_BIN_EXE_breakfree-docs"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!("BreakFree Docs {VERSION}\n")
    );
}
#[test]
fn malformed_open_and_bad_arguments_fail_before_a_window() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.bfdoc");
    std::fs::write(&path, b"broken").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_breakfree-docs"))
        .args([OsString::from("--open"), path.clone().into_os_string()])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert_eq!(std::fs::read(path).unwrap(), b"broken");
    let bad = Command::new(env!("CARGO_BIN_EXE_breakfree-docs"))
        .args(["--new", "--version"])
        .output()
        .unwrap();
    assert_eq!(bad.status.code(), Some(2));
}

#[test]
fn information_and_rejected_arguments_do_not_read_or_create_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let profile = dir.path().join("profile");
    for exists in [false, true] {
        if exists {
            std::fs::create_dir(&profile).unwrap();
            std::fs::write(profile.join("settings.json"), b"unknown future settings").unwrap();
        }
        for args in [
            vec!["--version"],
            vec!["--capabilities"],
            vec!["--new", "--version"],
            vec!["--open", "relative.bfdoc"],
        ] {
            let result = Command::new(env!("CARGO_BIN_EXE_breakfree-docs"))
                .args(&args)
                .current_dir(dir.path())
                .env("BREAKFREE_DOCS_CONFIG_DIR", &profile)
                .env_remove("DISPLAY")
                .env_remove("WAYLAND_DISPLAY")
                .output()
                .unwrap();
            if args.len() == 1 {
                assert!(result.status.success());
                assert!(result.stderr.is_empty());
            } else {
                assert_eq!(result.status.code(), Some(2));
            }
            if exists {
                assert_eq!(
                    std::fs::read(profile.join("settings.json")).unwrap(),
                    b"unknown future settings"
                );
                assert_eq!(std::fs::read_dir(&profile).unwrap().count(), 1);
            } else {
                assert!(!profile.exists());
            }
        }
    }
}
