use crate::parse2::SourceLocation;
use std::ops::Range;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Source<'s> {
    pub(crate) file_name: String,
    pub(crate) inner: ariadne::Source<&'s str>,
}

impl<'s> Source<'s> {
    pub(crate) fn new(file_name: &str, text: &'s str) -> Self {
        Self {
            file_name: file_name.to_owned(),
            inner: ariadne::Source::from(text),
        }
    }

    pub(super) fn location(&'s self, line: usize, columns: Range<usize>) -> SourceLocation<'s> {
        SourceLocation::new(self, line, columns)
    }

    pub(super) fn full_line(&'s self, line: usize) -> SourceLocation<'s> {
        SourceLocation::full_line(self, line)
    }
}