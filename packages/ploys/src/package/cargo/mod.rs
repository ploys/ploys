//! The `Cargo.toml` package for Rust.

mod dependency;
mod error;
mod lockfile;
mod manifest;

pub use self::dependency::{Dependencies, DependenciesMut, Dependency, DependencyMut};
pub use self::error::Error;
pub use self::lockfile::CargoLockfile;
pub use self::manifest::CargoManifest;
