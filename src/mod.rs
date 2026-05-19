#[cfg(feature = "analyzer")]
mod analyzer;

mod tokenizer;
#[cfg(not(feature = "analyzer"))]
mod parser;

#[cfg(all(not(feature = "analyzer"), not(feature = "wasm")))]
mod logger;

#[cfg(not(feature = "analyzer"))]
mod packages;