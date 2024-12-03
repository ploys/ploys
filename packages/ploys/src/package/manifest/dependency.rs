use semver::Version;

use super::cargo::{
    Dependencies as CargoDependencies, DependenciesMut as CargoDependenciesMut,
    Dependency as CargoDependency, DependencyMut as CargoDependencyMut,
};

/// The package dependency.
#[derive(Clone, Debug)]
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

    /// Gets the dependency version if it has been set.
    pub fn version(&self) -> Option<&'a str> {
        match self {
            Self::Cargo(dependency) => dependency.version(),
        }
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&'a str> {
        match self {
            Self::Cargo(dependency) => dependency.path(),
        }
    }
}

/// The package dependencies.
#[derive(Clone, Debug)]
pub enum Dependencies<'a> {
    Cargo(CargoDependencies<'a>),
}

impl<'a> Dependencies<'a> {
    /// Gets the dependency with the given name.
    pub fn get(&self, name: impl AsRef<str>) -> Option<Dependency<'a>> {
        match self {
            Self::Cargo(dependencies) => dependencies.get(name).map(Dependency::Cargo),
        }
    }
}

impl<'a> IntoIterator for Dependencies<'a> {
    type Item = Dependency<'a>;
    type IntoIter = Box<dyn Iterator<Item = Dependency<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Cargo(dependencies) => Box::new(dependencies.into_iter().map(Dependency::Cargo)),
        }
    }
}

/// The mutable package dependency.
#[derive(Debug)]
pub enum DependencyMut<'a> {
    /// A cargo package dependency.
    Cargo(CargoDependencyMut<'a>),
}

impl DependencyMut<'_> {
    /// Gets the dependency name.
    pub fn name(&self) -> &str {
        match self {
            Self::Cargo(dependency) => dependency.name(),
        }
    }

    /// Gets the dependency version if it has been set.
    pub fn version(&self) -> Option<Version> {
        match self {
            Self::Cargo(dependency) => dependency.version(),
        }
    }

    /// Sets the dependency version.
    pub fn set_version(&mut self, version: impl Into<Version>) {
        match self {
            Self::Cargo(dependency) => dependency.set_version(version),
        }
    }

    /// Gets the dependency path if it has been set.
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Cargo(dependency) => dependency.path(),
        }
    }
}

/// The mutable package dependencies.
#[derive(Debug)]
pub enum DependenciesMut<'a> {
    Cargo(CargoDependenciesMut<'a>),
}

impl DependenciesMut<'_> {
    /// Gets the mutable dependency with the given name.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<DependencyMut<'_>> {
        match self {
            Self::Cargo(dependencies) => dependencies.get_mut(name).map(DependencyMut::Cargo),
        }
    }
}

impl<'a> IntoIterator for DependenciesMut<'a> {
    type Item = DependencyMut<'a>;
    type IntoIter = Box<dyn Iterator<Item = DependencyMut<'a>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Cargo(dependencies) => {
                Box::new(dependencies.into_iter().map(DependencyMut::Cargo))
            }
        }
    }
}
