pub mod bytes;

#[cfg(not(feature = "analyzer"))]
pub(super) mod parser;
#[cfg(not(feature = "analyzer"))]
pub mod structure;
#[cfg(all(test, not(feature = "analyzer")))]
pub(crate) mod testing;
