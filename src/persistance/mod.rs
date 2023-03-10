use std::error;

pub mod aof;

#[derive(PartialEq, Debug)]
pub enum Type {
    None,
    Aof,
}

// Abstracts the layer for persistance
// Currently only AOF implements the trait
pub trait Persist {
    fn set(&self, key: &String, val: &String);
    fn del(&self, key: &String);
}

pub struct Empty;

impl Persist for Empty {
    fn set(&self, key: &String, val: &String) {}
    fn del(&self, key: &String) {}
}

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
