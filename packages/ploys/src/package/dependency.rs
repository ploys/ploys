use super::cargo::Dependency as CargoDependency;

/// The package dependency.
#[derive(Debug)]
pub enum Dependency<'a> {
    /// A cargo package dependency.
    Cargo(CargoDependency<'a>),
}

impl<'a> Dependency<'a> {
    /// Gets the dependency name.
    pub fn name(&self) -> &'a str {
        match self {
            Self::Cargo(dependency) => dependency.name(),
        }
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&'a str> {
        match self {
            Self::Cargo(dependency) => dependency.path(),
        }
    }
}
