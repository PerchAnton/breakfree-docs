use breakfree_docs::style_filter::Scope;
#[test]
fn scope_and_unicode_name_matching_are_independent() {
    assert!(Scope::All.matches("  НОТАТКА ", "Нотатка Україна", false, true));
    assert!(Scope::Custom.matches("рус", "Русский стиль", true, true));
    assert!(Scope::Character.matches("STRONG", "Strong", true, false));
    assert!(!Scope::Paragraph.matches("", "Strong", true, false));
    assert!(!Scope::Custom.matches("", "Normal", false, false));
    assert!(Scope::Paragraph.matches("e\u{301}", "cafe\u{301}", false, false));
    assert!(!Scope::All.matches("é", "cafe\u{301}", false, false));
    assert!(Scope::All.matches("", "Normal", false, false));
    assert!(!Scope::All.matches("absent", "Normal", false, false));
    let mut scope = Scope::All;
    for _ in 0..4 {
        scope = scope.next();
    }
    assert_eq!(scope, Scope::All);
}
