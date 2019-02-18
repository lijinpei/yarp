#![feature(hash_raw_entry)]

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
pub mod lexer;
