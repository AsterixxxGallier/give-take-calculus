use crate::parse::source::Source;
use crate::parse::SourceLocation;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct SourceLocationLines<'s> {
    pub(crate) file: &'s Source<'s>,
    /// inclusive start bound
    pub(crate) start_line: usize,
    /// exclusive end bound
    pub(crate) end_line: usize,
    pub(crate) reference_indentation: Option<SourceLocation<'s>>,
}

impl<'s> SourceLocationLines<'s> {
    pub(super) fn top_level(file: &'s Source<'s>) -> Self {
        Self {
            file,
            start_line: 0,
            end_line: file.inner.lines().len(),
            reference_indentation: None,
        }
    }

    /// Number of lines spanned by this location.
    pub(crate) fn len(&self) -> usize {
        self.end_line - self.start_line
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.start_line == self.end_line
    }

    pub(crate) fn first(&self) -> Option<SourceLocation<'s>> {
        if self.is_empty() {
            None
        } else {
            Some(self.file.full_line(self.start_line))
        }
    }

    /// Advances `self.start_line` by one and returns `Some(self)` if possible, and returns `None`
    /// if the location is already empty.
    pub(crate) fn advance(mut self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            self.start_line += 1;
            Some(self)
        }
    }

    pub(crate) fn with_reference_indentation(
        mut self,
        reference_indentation: SourceLocation<'s>,
    ) -> Self {
        self.reference_indentation = Some(reference_indentation);
        self
    }
}
