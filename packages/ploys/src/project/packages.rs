use std::iter::FusedIterator;
use std::path::PathBuf;

use strum::IntoEnumIterator;

use crate::file::File;
use crate::package::manifest::Members;
use crate::package::{Manifest, Package, PackageKind};

use super::Project;

/// An iterator over packages in a project.
pub struct Packages<'a> {
    state: State<'a>,
}

impl<'a> Packages<'a> {
    /// Constructs a new packages iterator.
    pub(super) fn new(project: &'a Project) -> Self {
        Self {
            state: State::Initial { project },
        }
    }
}

impl<'a> Iterator for Packages<'a> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.state {
                State::Initial { project } => {
                    let mut kinds = PackageKind::iter();

                    match kinds.next() {
                        Some(kind) => match project.get_file(kind.file_name()) {
                            Some(File::Manifest(manifest)) => {
                                self.state = State::Manifest {
                                    packages: ManifestPackages {
                                        project,
                                        manifest,
                                        members: match manifest.members() {
                                            Ok(members) => members,
                                            Err(_) => continue,
                                        },
                                        files: project.get_file_index().iter(),
                                    },
                                    kinds,
                                };
                                continue;
                            }
                            _ => continue,
                        },
                        None => break None,
                    }
                }
                State::Manifest { packages, kinds } => match packages.next() {
                    Some(package) => break Some(package),
                    None => match kinds.next() {
                        Some(kind) => packages.next_kind(kind),
                        None => break None,
                    },
                },
            }
        }
    }
}

enum State<'a> {
    Initial {
        project: &'a Project,
    },
    Manifest {
        packages: ManifestPackages<'a>,
        kinds: <PackageKind as IntoEnumIterator>::Iterator,
    },
}

/// An iterator over packages in a package manifest.
struct ManifestPackages<'a> {
    project: &'a Project,
    manifest: &'a Manifest,
    members: Members,
    files: std::collections::btree_set::Iter<'a, PathBuf>,
}

impl ManifestPackages<'_> {
    /// Advances the iterator to the next kind.
    fn next_kind(&mut self, kind: PackageKind) {
        let Some(File::Manifest(manifest)) = self.project.get_file(kind.file_name()) else {
            return;
        };

        self.manifest = manifest;
        self.files = self.project.get_file_index().iter();
    }
}

impl<'a> Iterator for ManifestPackages<'a> {
    type Item = Package<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let path = self.files.next()?;
            let manifest_path = self.manifest.package_kind().file_name();

            if path.file_name() != Some(manifest_path.as_os_str()) {
                continue;
            }

            let Some(parent) = path.parent() else {
                continue;
            };

            if path != manifest_path && !self.members.includes(parent) {
                continue;
            }

            let Some(File::Manifest(manifest)) = self.project.get_file(path) else {
                continue;
            };

            let Some(package) = Package::from_manifest(self.project, path, manifest) else {
                continue;
            };

            break Some(package);
        }
    }
}

impl FusedIterator for ManifestPackages<'_> {}
