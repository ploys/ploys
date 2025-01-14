use std::borrow::Cow;
use std::iter::FusedIterator;
use std::path::Path;

use strum::IntoEnumIterator;

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

impl Iterator for Packages<'_> {
    type Item = Package;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.state {
                State::Initial { project } => {
                    let kind = self.kinds.next()?;
                    let manifest = project
                        .repository
                        .get_file(kind.file_name())
                        .ok()
                        .flatten()
                        .and_then(|bytes| Manifest::from_bytes(kind, &bytes).ok());

                    if let Some(manifest) = manifest {
                        if let Ok(members) = manifest.members() {
                            if let Ok(files) = project.repository.get_file_index() {
                                self.state = State::Manifest {
                                    packages: ManifestPackages {
                                        project,
                                        manifest,
                                        members,
                                        files,
                                    },
                                };
                            }
                        }
                    }
                }
                State::Manifest { packages } => match packages.next() {
                    Some(package) => break Some(package),
                    None => {
                        let kind = self.kinds.next()?;
                        let manifest = packages
                            .project
                            .repository
                            .get_file(kind.file_name())
                            .ok()
                            .flatten()
                            .and_then(|bytes| Manifest::from_bytes(kind, &bytes).ok());

                        if let Some(manifest) = manifest {
                            if let Ok(members) = manifest.members() {
                                if let Ok(files) = packages.project.repository.get_file_index() {
                                    packages.manifest = manifest;
                                    packages.members = members;
                                    packages.files = files;
                                }
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
    manifest: Manifest,
    members: Members,
    files: Box<dyn Iterator<Item = Cow<'a, Path>> + 'a>,
}

impl Iterator for ManifestPackages<'_> {
    type Item = Package;

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

            let manifest = self
                .project
                .repository
                .get_file(&path)
                .ok()
                .flatten()
                .and_then(|bytes| Manifest::from_bytes(self.manifest.package_kind(), &bytes).ok());

            let Some(manifest) = manifest else {
                continue;
            };

            let Some(package) = Package::from_manifest(self.project, parent, manifest) else {
                continue;
            };

            break Some(package);
        }
    }
}

impl FusedIterator for ManifestPackages<'_> {}
