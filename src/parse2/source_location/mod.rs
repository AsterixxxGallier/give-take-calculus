use crate::parse2::source::Source;
use ariadne::{Color, Label, Report, ReportKind};
use std::ops::Range;
use std::str::pattern::{DoubleEndedSearcher, Pattern, ReverseSearcher, SearchStep, Searcher};

#[cfg(test)]
mod test;

fn source_line<'s>(source: &'s Source<'s>, index: usize) -> &'s str {
    source.inner.get_line_text(source.inner.line(index).unwrap()).unwrap()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct SourceLocation<'s> {
    pub(crate) file: &'s Source<'s>,
    pub(crate) line: usize,
    // we can't use std::ops::Range here because it's not Copy
    /// inclusive start bound
    pub(crate) start_column: usize,
    /// exclusive start bound
    pub(crate) end_column: usize,
    pub(crate) line_offset: usize,
}

impl<'s> ariadne::Span for SourceLocation<'s> {
    type SourceId = String;

    fn source(&self) -> &Self::SourceId {
        &self.file.file_name
    }

    fn start(&self) -> usize {
        self.line_offset + self.start_column
    }

    fn end(&self) -> usize {
        self.line_offset + self.end_column
    }
}

impl<'s> SourceLocation<'s> {
    pub(super) fn new(file: &'s Source<'s>, line: usize, columns: Range<usize>) -> Self {
        assert!(columns.end <= source_line(file, line).len());
        Self {
            file,
            line,
            start_column: columns.start,
            end_column: columns.end,
            // maybe this should be the byte offset instead, but that's private :(
            // same for Self::full_line
            line_offset: file.inner.line(line).unwrap().offset(),
        }
    }

    pub(super) fn full_line(file: &'s Source<'s>, line: usize) -> Self {
        Self {
            file,
            line,
            start_column: 0,
            end_column: source_line(file, line).len(),
            line_offset: file.inner.line(line).unwrap().offset(),
        }
    }

    pub(crate) fn dump(self, message: &str) {
        Report::build(ReportKind::Custom("Dump", Color::Cyan), self)
            .with_label(
                Label::new(self)
                    .with_color(Color::Green)
                    .with_message("here"),
            )
            .with_message(message)
            .finish()
            .eprint((self.file.file_name.clone(), &self.file.inner))
            .unwrap();
    }

    fn line_str(&self) -> &'s str {
        source_line(self.file, self.line)
    }

    pub(crate) fn as_str(&self) -> &'s str {
        &self.line_str()[self.start_column..self.end_column]
    }

    pub(crate) fn len(&self) -> usize {
        self.end_column - self.start_column
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.start_column == self.end_column
    }

    pub(super) fn truncate(mut self, max_len: usize) -> Self {
        if self.len() > max_len {
            self.end_column = self.start_column + max_len;
        }
        self
    }

    pub(super) fn grow(mut self, by: usize) -> Self {
        assert!(self.start_column >= by, "can't grow source location");
        assert!(
            self.end_column <= self.line_str().len() - by,
            "can't grow source location"
        );
        self.start_column -= by;
        self.end_column += by;
        self
    }

    pub(super) fn grow_start(mut self, by: usize) -> Self {
        assert!(self.start_column >= by, "can't grow source location");
        self.start_column -= by;
        self
    }

    pub(super) fn grow_end(mut self, by: usize) -> Self {
        assert!(
            self.end_column <= self.line_str().len() - by,
            "can't grow source location"
        );
        self.end_column += by;
        self
    }

    pub(super) fn starts_with(&self, pattern: impl Pattern) -> bool {
        self.as_str().starts_with(pattern)
    }

    pub(super) fn trim_start(self) -> Self {
        self.trim_start_matches(char::is_whitespace)
    }

    pub(super) fn trim(self) -> Self {
        self.trim_matches(char::is_whitespace)
    }

    pub(super) fn take_while_whitespace(self) -> Self {
        let (word, _rest) = self.partition(char::is_whitespace);
        word
    }

    pub(super) fn take_until_whitespace(self) -> Self {
        fn is_not_whitespace(char: char) -> bool {
            !char.is_whitespace()
        }

        let (word, _rest) = self.partition(is_not_whitespace);
        word
    }

    /// Splits this location in two: the part for which `pattern` (applied repeatedly) matches and
    /// the remainder of the location.
    pub(super) fn partition(self, pattern: impl Pattern) -> (Self, Self) {
        let mut searcher = pattern.into_searcher(self.as_str());
        let mut matching_part = self;
        let mut rejected_part = self;
        if let Some((start, _)) = searcher.next_reject() {
            // the start of the rejection is the index of the first char that does not match the
            // pattern
            // start is relative to self.as_str(), hence += instead of +
            rejected_part.start_column += start;
            matching_part.end_column = matching_part.start_column + start;
        } else {
            rejected_part.start_column = rejected_part.end_column;
        }
        (matching_part, rejected_part)
    }

    pub(super) fn trim_start_matches(mut self, pattern: impl Pattern) -> Self {
        let mut searcher = pattern.into_searcher(self.as_str());
        if let Some((start, _)) = searcher.next_reject() {
            // the start of the rejection is the index of the first char that does not match the
            // pattern
            // start is relative to self.as_str(), hence += instead of +
            self.start_column += start;
        } else {
            self.start_column = self.end_column;
        }
        self
    }

    pub(super) fn trim_matches<P: Pattern>(mut self, pattern: P) -> Self
    where
            for<'a> P::Searcher<'a>: DoubleEndedSearcher<'a>,
    {
        let mut searcher = pattern.into_searcher(self.as_str());

        let mut new_start = 0;
        let mut new_end = 0;
        if let Some((start, end)) = searcher.next_reject() {
            new_start = start;
            new_end = end;
        }
        if let Some((_, b)) = searcher.next_reject_back() {
            new_end = b;
        }

        // new_start and new_end are relative to self.as_str()
        let old_start = self.start_column;
        self.start_column = old_start + new_start;
        self.end_column = old_start + new_end;

        self
    }

    pub(super) fn strip_prefix<P: Pattern>(mut self, pattern: P) -> Option<Self> {
        let mut searcher = pattern.into_searcher(self.as_str());
        if let SearchStep::Match(_, len) = searcher.next() {
            self.start_column += len;
            Some(self)
        } else {
            None
        }
    }
}
