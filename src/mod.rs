#[cfg(feature = "analyzer")]
mod analyzer;

mod tokenizer;
#[cfg(not(feature = "analyzer"))]
mod parser;

#[cfg(not(target_family = "wasm"))]
mod logger;

#[cfg(not(feature = "analyzer"))]
mod packages;