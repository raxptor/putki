extern crate seahash;

mod inki;
mod pipeline;
mod shared;

pub mod outki;
pub use shared::*;
pub use pipeline::*;
pub use writer::*;
pub use inki::*;

#[cfg(test)]
mod tests;

pub trait Mixki { }
pub trait Inki { }
pub trait Outki { }
