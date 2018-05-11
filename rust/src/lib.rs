pub mod inki;
pub mod outki;
pub mod pipeline;
pub mod shared;

#[cfg(test)]
mod tests;

pub trait Mixki { }
pub trait Inki { }
pub trait Outki { }