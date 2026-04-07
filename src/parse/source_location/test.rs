use crate::parse::{Source, SourceLocation};

#[test]
fn grow() {
    let source = Source::new("foo.txt", "hello");
    let location = SourceLocation::new(&source, 0, 2..3);
    let full_line = SourceLocation::full_line(&source, 0);
    assert_eq!(location.grow(2), full_line);
}

#[test]
#[should_panic]
fn grow_out_of_bounds() {
    let source = Source::new("foo.txt", "hello");
    let location = SourceLocation::new(&source, 0, 2..3);
    _ = location.grow(3);
}

#[test]
fn trim_start() {
    let source = Source::new("foo.txt", "   hello");
    let full_line = SourceLocation::full_line(&source, 0);
    let hello = SourceLocation::new(&source, 0, 3..8);
    assert_eq!(full_line.trim_start(), hello);
}

#[test]
fn trim() {
    let source = Source::new("foo.txt", "   hello  ");
    let full_line = SourceLocation::full_line(&source, 0);
    let hello = SourceLocation::new(&source, 0, 3..8);
    assert_eq!(full_line.trim(), hello);
}

#[test]
fn truncate_to_word() {
    let source = Source::new("foo.txt", "hello foo bar");
    let full_line = SourceLocation::full_line(&source, 0);
    let hello = SourceLocation::new(&source, 0, 0..5);
    assert_eq!(full_line.take_until_whitespace(), hello);
}

#[test]
fn strip_prefix() {
    let source = Source::new("foo.txt", "hello foo bar");
    let full_line = SourceLocation::full_line(&source, 0);
    let after_hello = SourceLocation::new(&source, 0, 5..13);
    assert_eq!(full_line.strip_prefix("hello"), Some(after_hello));
    assert_eq!(full_line.strip_prefix("not hello"), None);
}
