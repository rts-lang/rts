pub mod bytes;
#[cfg(not(feature = "analyzer"))]
pub(super) mod parser;
#[cfg(not(feature = "analyzer"))]
pub mod structure;
// =================================================================================================
/// Это модуль хелперов для тестов.
#[cfg(test)]
pub(super) mod testing;
/// Это модуль для тестов.
#[cfg(test)]
mod tests;
// =================================================================================================
