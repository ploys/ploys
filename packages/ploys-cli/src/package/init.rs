use std::io::IsTerminal;
use std::path::PathBuf;

use anyhow::{Error, bail};
use clap::{Args, ValueEnum};
use dialoguer::{Input, MultiSelect, Select};
use itertools::Itertools;
use ploys::changelog::Changelog;
use ploys::package::Package;
use ploys::project::Project;
use ploys::repository::types::fs::FileSystem;
use strum::{Display, VariantArray};

/// The `package init` command.
#[derive(Args)]
pub struct Init {
    /// The package name.
    name: Option<String>,

    /// The package description.
    #[arg(long)]
    description: Option<String>,

    /// The package author.
    #[arg(long)]
    author: Vec<String>,

    /// The package type.
    #[arg(long)]
    r#type: Option<PackageType>,

    /// Creates the package with a binary target.
    #[arg(long)]
    bin: bool,

    /// Creates the package with a library target.
    #[arg(long)]
    lib: bool,

    /// The project path.
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

impl Init {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let mut project = Project::fs(&self.path)?;
        let package = self.build_package(&project)?;

        project.add_package(package)?;
        project.write()?;

        Ok(())
    }

    fn get_name(&self) -> Result<String, Error> {
        match &self.name {
            Some(name) => Ok(name.clone()),
            None if !std::io::stderr().is_terminal() => bail!("Expected a package name"),
            None => Ok(Input::<String>::new().with_prompt("Name").interact_text()?),
        }
    }

    fn get_description(&self) -> Result<Option<String>, Error> {
        match &self.description {
            Some(description) => Ok(Some(description.clone())),
            None if !std::io::stderr().is_terminal() => Ok(None),
            None => {
                let description = Input::<String>::new()
                    .with_prompt("Description")
                    .allow_empty(true)
                    .interact_text()?;

                match description.is_empty() {
                    true => Ok(None),
                    false => Ok(Some(description)),
                }
            }
        }
    }

    fn get_authors(&self, project: &Project<FileSystem>) -> Result<Vec<String>, Error> {
        match self.author.is_empty() {
            false => Ok(self.author.clone()),
            true if !std::io::stderr().is_terminal() => Ok(Vec::new()),
            true if project.authors().count() == 0 => {
                let authors = Input::<String>::new()
                    .with_prompt("Authors")
                    .allow_empty(true)
                    .interact_text()?;

                match authors.is_empty() {
                    true => Ok(Vec::new()),
                    false => Ok(authors
                        .split(",")
                        .map(str::trim)
                        .map(ToOwned::to_owned)
                        .collect()),
                }
            }
            true => {
                let authors = Input::<String>::new()
                    .with_prompt("Authors")
                    .with_initial_text(project.authors().join(", "))
                    .allow_empty(true)
                    .interact_text()?;

                match authors.is_empty() {
                    true => Ok(Vec::new()),
                    false => Ok(authors
                        .split(",")
                        .map(str::trim)
                        .map(ToOwned::to_owned)
                        .collect()),
                }
            }
        }
    }

    fn get_package_type(&self) -> Result<PackageType, Error> {
        match self.r#type {
            Some(package_type) => Ok(package_type),
            None if !std::io::stderr().is_terminal() => bail!("Expected a package type"),
            None => {
                let selection = Select::new()
                    .with_prompt("Type")
                    .items(PackageType::VARIANTS)
                    .default(0)
                    .interact()?;

                Ok(PackageType::VARIANTS[selection])
            }
        }
    }

    fn get_package_targets(&self) -> Result<Vec<PackageTarget>, Error> {
        let mut targets = Vec::new();

        if !std::io::stderr().is_terminal() {
            if !self.bin && !self.lib {
                bail!("Expected either a binary or library target");
            }

            if self.bin {
                targets.push(PackageTarget::Binary);
            }

            if self.lib {
                targets.push(PackageTarget::Library);
            }
        } else if !self.bin && !self.lib {
            let selection = MultiSelect::new()
                .with_prompt("Targets")
                .items(PackageTarget::VARIANTS)
                .defaults(&[true, false])
                .interact()?;

            if selection.is_empty() {
                bail!("Expected either a binary or library target");
            }

            for index in selection {
                targets.push(PackageTarget::VARIANTS[index]);
            }
        } else {
            if self.bin {
                targets.push(PackageTarget::Binary);
            }

            if self.lib {
                targets.push(PackageTarget::Library);
            }
        }

        Ok(targets)
    }

    fn build_package(&self, project: &Project<FileSystem>) -> Result<Package, Error> {
        let name = self.get_name()?;
        let description = self.get_description()?;
        let authors = self.get_authors(project)?;
        let package_type = self.get_package_type()?;

        let mut package = match package_type {
            PackageType::Cargo => Package::new_cargo(name),
        };

        if let Some(description) = description {
            package.set_description(description);
        }

        if !authors.is_empty() {
            for author in authors {
                package.add_author(author);
            }
        }

        if let Some(repo) = project.repository() {
            package.set_repository(repo.to_url());
        }

        #[allow(irrefutable_let_patterns)]
        if let PackageType::Cargo = package_type {
            let package_targets = self.get_package_targets()?;

            if package_targets.contains(&PackageTarget::Library) {
                package.add_file("src/lib.rs", "\n")?;
            }

            if package_targets.contains(&PackageTarget::Binary) || package_targets.is_empty() {
                package.add_file(
                    "src/main.rs",
                    "fn main() {\n    println!(\"Hello, world!\");\n}\n",
                )?;
            }

            package.add_file("CHANGELOG.md", Changelog::new().to_string())?;
        }

        Ok(package)
    }
}

#[derive(Clone, Copy, Debug, Display, VariantArray, ValueEnum)]
enum PackageType {
    Cargo,
}

#[derive(Clone, Copy, Debug, Display, PartialEq, VariantArray)]
enum PackageTarget {
    Binary,
    Library,
}
