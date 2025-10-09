#![allow(unexpected_cfgs)]

pub mod application;
pub mod config;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;

pub use error::{AppResult, ConfigError};

#[cfg(test)]
pub(crate) mod test_support;
