/// Prepare a deliberately older fixture from the current encoder, so legacy
/// tests continue to exercise their own version's fields rather than v19 gates.
pub fn strip_structure_for_legacy(value: &mut serde_json::Value) {
    if value["content_version"].as_u64().is_some_and(|v| v < 19) {
        value.as_object_mut().unwrap().remove("page");
        if let Some(paragraphs) = value["paragraphs"].as_array_mut() {
            for p in paragraphs {
                p.as_object_mut().unwrap().remove("structure");
            }
        }
    }
}
