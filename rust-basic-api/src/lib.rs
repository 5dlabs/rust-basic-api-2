#![allow(unexpected_cfgs)]

pub mod application;
pub mod config;
pub mod error;
pub mod models;
pub mod repository;
pub mod routes;

pub use application::run;
pub use error::{AppResult, ConfigError};
