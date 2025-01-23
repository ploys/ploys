use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Error};
use clap::{Args, ValueEnum};
use dialoguer::{Input, Select};
use ploys::changelog::Changelog;
use ploys::package::Package;
use ploys::project::Project;
use ploys::repository::RepoSpec;
use strum::{Display, VariantArray};

/// Initializes a new project.
#[derive(Args)]
pub struct Init {
    /// The target path.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// The project name.
    #[arg(long)]
    name: Option<String>,

    /// The project description.
    #[arg(long)]
    description: Option<String>,

    /// The project repository.
    #[arg(long)]
    repository: Option<RepoSpec>,

    /// The project template.
    #[arg(long, value_enum)]
    template: Option<Template>,
}

impl Init {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let is_terminal = std::io::stderr().is_terminal();

        let name = match self.name {
            Some(name) => name,
            None if self.path != Path::new(".") => match self.path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None if !is_terminal => bail!("Expected a project name"),
                None => Input::<String>::new().with_prompt("Name").interact_text()?,
            },
            None if !is_terminal => bail!("Expected a project name"),
            None => Input::<String>::new().with_prompt("Name").interact_text()?,
        };

        let description = match self.description {
            Some(description) => Some(description),
            None if !is_terminal => None,
            None => {
                let description = Input::<String>::new()
                    .with_prompt("Description")
                    .allow_empty(true)
                    .interact_text()?;

                match description.is_empty() {
                    true => None,
                    false => Some(description),
                }
            }
        };

        let repository = match self.repository {
            Some(repository) => Some(repository),
            None if !is_terminal => None,
            None => {
                let repository = Input::<String>::new()
                    .with_prompt("Repository")
                    .allow_empty(true)
                    .interact_text()?;

                match repository.is_empty() {
                    true => None,
                    false => Some(repository.parse()?),
                }
            }
        };

        let template = match self.template {
            Some(template) => template,
            None if !is_terminal => Template::None,
            None => {
                let selection = Select::new()
                    .with_prompt("Template")
                    .items(Template::VARIANTS)
                    .default(0)
                    .interact()?;

                Template::VARIANTS[selection]
            }
        };

        let mut project = Project::new(&name);

        if let Some(description) = description {
            project.set_description(description);
        }

        if let Some(repository) = repository {
            project.set_repository(repository);
        }

        match template {
            Template::CargoBin => {
                let mut package = Package::new_cargo(name);

                if let Some(description) = project.description() {
                    package.set_description(description);
                }

                if let Some(repository) = project.repository() {
                    package.set_repository(repository.to_url());
                }

                package.add_file(
                    "src/main.rs",
                    b"fn main() {\n    println!(\"Hello, world!\");\n}\n",
                );
                package.add_file("CHANGELOG.md", Changelog::new().to_string().into_bytes());
                project.add_package(package)?;
            }
            Template::CargoLib => {
                let mut package = Package::new_cargo(name);

                if let Some(description) = project.description() {
                    package.set_description(description);
                }

                if let Some(repository) = project.repository() {
                    package.set_repository(repository.to_url());
                }

                package.add_file("src/lib.rs", b"\n");
                package.add_file("CHANGELOG.md", Changelog::new().to_string().into_bytes());
                project.add_package(package)?;
            }
            Template::None => {}
        }

        if !self.path.exists() {
            if self.path.is_relative() {
                std::fs::create_dir_all(&self.path).with_context(|| {
                    format!("Could not create directory `{}`", self.path.display())
                })?;
            } else {
                std::fs::create_dir(&self.path).with_context(|| {
                    format!("Could not create directory `{}`", self.path.display())
                })?;
            }
        }

        project.write(&self.path, false).with_context(|| {
            format!(
                "Could not create project at directory `{}`",
                self.path.display()
            )
        })?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Display, VariantArray, ValueEnum)]
enum Template {
    #[strum(to_string = "Cargo (binary)")]
    CargoBin,
    #[strum(to_string = "Cargo (library)")]
    CargoLib,
    #[strum(to_string = "None")]
    None,
}
