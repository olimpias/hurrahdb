use std::error;

pub mod aof;

// Abstracts the layer for persistance
// Currently only AOF implements the trait
pub trait Persist {
    fn set(&self, key: &str, val: &str);
    fn del(&self, key: &str);
}

pub struct Empty;

impl Persist for Empty {
    fn set(&self, _key: &str, _val: &str) {}
    fn del(&self, _key: &str) {}
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
