use std::fmt::{Debug, Formatter};

pub(crate) struct Source<'s> {
    pub(super) file_name: String,
    pub(super) text: &'s str,
    pub(super) lines: Vec<&'s str>,
}

impl<'s> Source<'s> {
    pub(crate) fn new(file_name: &str, text: &'s str) -> Self {
        Self {
            file_name: file_name.to_owned(),
            text,
            lines: text.lines().collect(),
        }
    }
}

impl<'s> Debug for Source<'s> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.file_name)
    }
}

impl<'s> PartialEq for Source<'s> {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
    }
}

impl<'s> Eq for Source<'s> {}
