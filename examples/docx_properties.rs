//! Original custom-property fixture; metadata remains passive and byte-preserved.
use breakfree_docs::{docx, storage};
fn main() -> Result<(), String> {
    let mut document = storage::load(std::path::Path::new("examples/DocxExtended.bfdoc"))?;
    document.package_properties.insert("custom".into(),
        "<p:Properties xmlns:p=\"http://schemas.openxmlformats.org/officeDocument/2006/custom-properties\" xmlns:vt=\"http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes\"><p:property fmtid=\"{D5CDD505-2E9C-101B-9397-08002B2CF9AE}\" pid=\"2\" name=\"Review label\"><vt:lpwstr>Original fixture · Україна · e&#x301;</vt:lpwstr></p:property><p:property fmtid=\"{D5CDD505-2E9C-101B-9397-08002B2CF9AE}\" pid=\"3\" name=\"Ready\"><vt:bool>false</vt:bool></p:property></p:Properties>".into());
    let docx_path = std::path::Path::new("examples/DocxProperties.docx");
    let native_path = std::path::Path::new("examples/DocxProperties.bfdoc");
    docx::save(&document, docx_path)?;
    storage::save(&document, native_path)?;
    assert_eq!(docx::load(docx_path)?, document);
    assert_eq!(storage::load(native_path)?, document);
    println!("Custom properties preserved across DOCX/native save and reopen.");
    Ok(())
}
