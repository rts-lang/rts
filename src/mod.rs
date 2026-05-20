#[cfg(feature = "analyzer")]
mod analyzer;

mod tokenizer;
#[cfg(not(feature = "analyzer"))]
mod parser;

#[cfg(not(feature = "wasm"))]
mod logger;

#[cfg(not(feature = "analyzer"))]
mod packages;