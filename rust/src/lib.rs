extern crate seahash;

mod inki;
mod outki;
mod pipeline;
mod shared;

pub use inki::*;
pub use outki::*;
pub use shared::*;
pub use pipeline::*;
pub use writer::*;

#[cfg(test)]
mod tests;

pub trait Mixki { }
pub trait Inki { }
pub trait Outki { }
