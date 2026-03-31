use std::fmt::{Debug, Display, Formatter};
use rand::random;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(super) struct Id(u64);

impl Id {
    pub(super) fn generate() -> Self {
        Self(random())
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0<4x}", self.0 & 0xffff)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0<4x}", self.0 & 0xffff)
    }
}