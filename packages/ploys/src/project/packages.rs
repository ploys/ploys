use std::iter::FusedIterator;
use std::path::PathBuf;

use strum::IntoEnumIterator;

use crate::file::File;
use crate::package::manifest::Members;
use crate::package::{Manifest, Package, PackageKind};

use super::Project;

/// An iterator over packages in a project.
pub struct Packages<'a> {
    kinds: <PackageKind as IntoEnumIterator>::Iterator,
    state: State<'a>,
}

impl<'a> Packages<'a> {
    /// Constructs a new packages iterator.
    pub(super) fn new(project: &'a Project) -> Self {
        Self {
            kinds: PackageKind::iter(),
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
                    let kind = self.kinds.next()?;

                    if let Some(File::Manifest(manifest)) = project.get_file(kind.file_name()) {
                        if let Ok(members) = manifest.members() {
                            self.state = State::Manifest {
                                packages: ManifestPackages {
                                    project,
                                    manifest,
                                    members,
                                    files: project.get_file_index().iter(),
                                },
                            };
                        }
                    }
                }
                State::Manifest { packages } => match packages.next() {
                    Some(package) => break Some(package),
                    None => {
                        let kind = self.kinds.next()?;

                        if let Some(File::Manifest(manifest)) =
                            packages.project.get_file(kind.file_name())
                        {
                            if let Ok(members) = manifest.members() {
                                packages.manifest = manifest;
                                packages.members = members;
                                packages.files = packages.project.get_file_index().iter();
                            }
                        }
                    }
                },
            }
        }
    }
}

impl FusedIterator for Packages<'_> {}

enum State<'a> {
    Initial { project: &'a Project },
    Manifest { packages: ManifestPackages<'a> },
}

/// An iterator over packages in a package manifest.
struct ManifestPackages<'a> {
    project: &'a Project,
    manifest: &'a Manifest,
    members: Members,
    files: std::collections::btree_set::Iter<'a, PathBuf>,
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
