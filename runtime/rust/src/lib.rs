#![feature(rc_downcast)] 

pub mod mixki;
pub mod pipeline;
pub mod loader;
pub mod inki;

//#[cfg(test)]
mod tests;

pub use mixki::lexer::*;
pub use mixki::parser::*;
pub use mixki::rtti::*;
pub use inki::*;

pub trait Mixki { }
pub trait Inki { }
pub trait Outki { }