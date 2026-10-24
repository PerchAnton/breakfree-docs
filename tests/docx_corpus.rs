//! Unmodified external-producer fixtures and an independent accepted baseline.
use breakfree_docs::{docx, session::Session};
use std::path::Path;

#[test]
fn external_corpus_acceptance_and_rejection_preserve_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docx-corpus");
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for case in manifest["cases"].as_array().unwrap() {
        let name = case["file"].as_str().unwrap();
        let source = std::fs::read(root.join(name)).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, &source).unwrap();
        if !case["accepted"].as_bool().unwrap() {
            let error = match Session::open(&path) {
                Ok(_) => panic!("Unexpectedly accepted {name}"),
                Err(e) => e,
            };
            assert!(
                error.contains(case["error_contains"].as_str().unwrap()),
                "{name}: {error}"
            );
            assert!(error.contains("word/_rels/document.xml.rels"), "{error}");
            assert_eq!(std::fs::read(&path).unwrap(), source);
            continue;
        }
        let mut session = Session::open(&path).unwrap();
        let original = session.editor.document.clone();
        assert_eq!(
            original.paragraphs[0].text,
            case["expected_text"].as_str().unwrap()
        );
        assert_eq!(original.paragraphs[0].styles[0].bold, Some(true));
        assert_eq!(
            original.paragraphs[0].styles.last().unwrap().italic,
            Some(true)
        );
        assert_eq!(
            original.paragraphs[1].alignment,
            breakfree_docs::model::Alignment::Center
        );
        session
            .editor
            .replace(0, 0, "Edited · ", "Corpus edit")
            .unwrap();
        let edited = session.editor.document.clone();
        session.editor.undo();
        assert_eq!(session.editor.document, original);
        session.editor.redo();
        assert_eq!(session.editor.document, edited);
        session.save(&path).unwrap();
        assert!(!session.dirty());
        assert_eq!(docx::load(&path).unwrap(), edited);
        session.editor.undo();
        assert!(session.dirty());
        session.save(&path).unwrap();
        assert_eq!(docx::load(&path).unwrap(), original);
        // The committed fixture is never overwritten by the save-back scenario.
        assert_eq!(std::fs::read(root.join(name)).unwrap(), source);
    }
}
