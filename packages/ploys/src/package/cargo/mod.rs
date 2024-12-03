//! The `Cargo.toml` package for Rust.

mod dependency;
mod error;

pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::error::Error;
