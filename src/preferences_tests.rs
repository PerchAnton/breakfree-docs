use super::*;

fn custom() -> Toolbar {
    Toolbar {
        commands: vec![QuickCommand::Open, QuickCommand::Save, QuickCommand::New],
        placement: Placement::Below,
    }
}
fn open(path: &Path) -> Store {
    Store::open(Ok(path.to_owned())).0
}

#[test]
fn profile_paths_and_overrides_are_absolute_and_do_not_require_unused_variables() {
    let root = tempfile::tempdir().unwrap();
    let base = root.path();
    for (os, variable, suffix) in [
        ("windows", "APPDATA", "BreakFree/Docs/settings.json"),
        (
            "macos",
            "HOME",
            "Library/Application Support/BreakFree/Docs/settings.json",
        ),
        ("linux", "XDG_CONFIG_HOME", "breakfree-docs/settings.json"),
        ("linux", "HOME", ".config/breakfree-docs/settings.json"),
    ] {
        assert_eq!(
            resolve_path(os, |name| (name == variable)
                .then(|| base.as_os_str().to_owned()))
            .unwrap(),
            base.join(suffix)
        );
    }
    assert_eq!(
        resolve_path("linux", |name| match name {
            "HOME" => Some(base.into()),
            "XDG_CONFIG_HOME" => Some("relative".into()),
            _ => None,
        })
        .unwrap(),
        base.join(".config/breakfree-docs/settings.json")
    );
    for os in ["windows", "macos", "linux"] {
        assert_eq!(
            resolve_path(os, |name| (name == "BREAKFREE_DOCS_CONFIG_DIR")
                .then(|| base.into()))
            .unwrap(),
            base.join("settings.json")
        );
        assert!(resolve_path(os, |_| None).is_err());
        for invalid in ["", "relative"] {
            assert!(
                resolve_path(os, |name| (name == "BREAKFREE_DOCS_CONFIG_DIR")
                    .then(|| invalid.into()))
                .is_err()
            );
        }
    }
    assert_eq!(fs::read_dir(base).unwrap().count(), 0);
}

#[test]
fn opening_does_not_create_files_and_changes_roundtrip_order_placement_and_empty_toolbar() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("profile/settings.json");
    let (mut store, toolbar, warning) = Store::open(Ok(path.clone()));
    assert_eq!(toolbar, Toolbar::default());
    assert!(warning.is_none());
    assert!(!path.parent().unwrap().exists());
    for toolbar in [
        custom(),
        Toolbar {
            commands: vec![],
            placement: Placement::Above,
        },
    ] {
        store.save(&toolbar).unwrap();
        let (reopened, restored, warning) = Store::open(Ok(path.clone()));
        assert_eq!(restored, toolbar);
        assert!(warning.is_none());
        store = reopened;
    }
    assert_eq!(fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn malformed_foreign_future_duplicate_and_unknown_settings_are_preserved_until_explicit_reset() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let valid = String::from_utf8(encode(&Toolbar::default()).unwrap()).unwrap();
    let cases = [
        b"{broken".to_vec(),
        vec![0xff],
        valid.replace(FORMAT, "foreign.settings").into_bytes(),
        valid
            .replace("\"settings_version\": 1", "\"settings_version\": 99")
            .into_bytes(),
        valid.replace("\"undo\"", "\"save\"").into_bytes(),
        valid.replace("\"undo\"", "\"print\"").into_bytes(),
        valid.replace("\"above\"", "\"side\"").into_bytes(),
        valid
            .replace("\"format\":", "\"extra\": true, \"format\":")
            .into_bytes(),
        valid
            .replace(
                "\"settings_version\": 1",
                "\"settings_version\": 1, \"settings_version\": 1",
            )
            .into_bytes(),
    ];
    for original in cases {
        fs::write(&path, &original).unwrap();
        let (mut store, toolbar, warning) = Store::open(Ok(path.clone()));
        assert_eq!(toolbar, Toolbar::default());
        assert!(warning.is_some());
        assert!(store.save(&custom()).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        let backup = store.reset().unwrap().unwrap();
        assert_eq!(fs::read(backup).unwrap(), original);
        assert_eq!(
            decode(&fs::read(&path).unwrap()).unwrap(),
            Toolbar::default()
        );
    }
}

#[test]
fn failed_partial_write_flush_and_replacement_keep_previous_settings_and_allow_retry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let original = encode(&Toolbar::default()).unwrap();
    for fail in [Stage::Write, Stage::Sync, Stage::Replace] {
        fs::write(&path, &original).unwrap();
        let mut store = open(&path);
        assert!(
            store
                .write(&custom(), false, |stage| if stage == fail {
                    Err("injected storage failure".into())
                } else {
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        store.save(&custom()).unwrap();
        assert_eq!(decode(&fs::read(&path).unwrap()).unwrap(), custom());
    }
}

#[test]
fn reset_never_replaces_original_if_backup_or_replacement_fails() {
    for fail in [Stage::Backup, Stage::Replace] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let original = b"unknown future settings";
        fs::write(&path, original).unwrap();
        let mut store = open(&path);
        assert!(
            store
                .write(&Toolbar::default(), true, |stage| if stage == fail {
                    Err("injected failure".into())
                } else {
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), original);
        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(files.len(), if fail == Stage::Replace { 2 } else { 1 });
        for file in files {
            assert_eq!(fs::read(file).unwrap(), original);
        }
    }
}

#[test]
fn concurrent_windows_and_busy_lock_do_not_lose_saved_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let mut first = open(&path);
    let mut second = open(&path);
    first.save(&custom()).unwrap();
    let saved = fs::read(&path).unwrap();
    assert!(
        second
            .save(&Toolbar::default())
            .unwrap_err()
            .contains("outside this window")
    );
    assert_eq!(fs::read(&path).unwrap(), saved);
    let lock = dir.path().join(".toolbar-settings.lock");
    fs::write(&lock, b"another writer").unwrap();
    assert!(
        first
            .save(&Toolbar::default())
            .unwrap_err()
            .contains("Cannot lock")
    );
    assert_eq!(fs::read(&lock).unwrap(), b"another writer");
    assert_eq!(fs::read(&path).unwrap(), saved);
    fs::remove_file(lock).unwrap();
    let backup = second.reset().unwrap().unwrap();
    assert_eq!(fs::read(backup).unwrap(), saved);
}

#[test]
fn oversized_non_file_and_unavailable_paths_are_never_overwritten() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let original = vec![b' '; MAX_BYTES as usize + 1];
    fs::write(&path, &original).unwrap();
    let (mut store, _, warning) = Store::open(Ok(path.clone()));
    assert!(warning.unwrap().contains("64 KiB"));
    assert!(store.save(&custom()).is_err());
    assert!(store.reset().is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    let blocked = dir.path().join("blocked");
    fs::write(&blocked, b"keep").unwrap();
    assert!(
        open(&blocked.join("settings.json"))
            .save(&custom())
            .is_err()
    );
    assert_eq!(fs::read(&blocked).unwrap(), b"keep");
    let (mut store, _, warning) = Store::open(Err("No profile path".into()));
    assert!(warning.is_some());
    assert!(store.save(&custom()).is_err());
    assert!(open(dir.path()).reset().is_err());
}

#[test]
fn readonly_settings_remain_intact_and_retry_works_after_permissions_restored() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let original = encode(&Toolbar::default()).unwrap();
    fs::write(&path, &original).unwrap();
    let permissions = fs::metadata(&path).unwrap().permissions();
    let mut readonly = permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&path, readonly).unwrap();
    let mut store = open(&path);
    assert!(store.save(&custom()).unwrap_err().contains("read-only"));
    assert!(store.reset().is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    fs::set_permissions(&path, permissions).unwrap();
    store.save(&custom()).unwrap();
    assert_eq!(decode(&fs::read(&path).unwrap()).unwrap(), custom());
}

#[cfg(unix)]
#[test]
fn symbolic_settings_links_are_refused_without_modifying_the_target() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target");
    let link = dir.path().join("settings.json");
    fs::write(&target, b"keep").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let (mut store, _, warning) = Store::open(Ok(link.clone()));
    assert!(warning.is_some());
    assert!(store.save(&custom()).is_err());
    assert!(store.reset().is_err());
    assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(target).unwrap(), b"keep");
}
