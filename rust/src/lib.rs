#![feature(rc_downcast)] 

pub mod inki;
pub mod pipeline;

#[cfg(test)]
mod tests;

pub trait Mixki { }
pub trait Inki { }
pub trait Outki { }