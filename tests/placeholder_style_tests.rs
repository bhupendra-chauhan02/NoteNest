use notenest::PlaceholderStyle;

#[test]
fn parses_placeholder_styles() {
    assert_eq!(
        PlaceholderStyle::from_str("protected"),
        Ok(PlaceholderStyle::Protected)
    );
    assert_eq!(
        PlaceholderStyle::from_str("masked"),
        Ok(PlaceholderStyle::Masked)
    );
    assert_eq!(
        PlaceholderStyle::from_str("hidden"),
        Ok(PlaceholderStyle::Hidden)
    );
    assert_eq!(
        PlaceholderStyle::from_str("removed"),
        Ok(PlaceholderStyle::Removed)
    );
    assert_eq!(
        PlaceholderStyle::from_str("angle"),
        Ok(PlaceholderStyle::Angle)
    );
}
