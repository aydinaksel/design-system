use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ELEMENT_NUMBER: AtomicU64 = AtomicU64::new(0);

const RENDERING_SIDE_PREFIX: &str = if cfg!(feature = "ssr") { "s" } else { "b" };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ElementKey(u64);

impl ElementKey {
    pub(crate) fn next() -> Self {
        Self(NEXT_ELEMENT_NUMBER.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for ElementKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{RENDERING_SIDE_PREFIX}{}", self.0)
    }
}
