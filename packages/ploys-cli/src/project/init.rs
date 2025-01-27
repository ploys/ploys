use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Error};
use clap::{Args, ValueEnum};
use dialoguer::{Input, Select};
use ploys::changelog::Changelog;
use ploys::package::Package;
use ploys::project::Project;
use ploys::repository::git::Git;
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

    /// The version control system.
    #[arg(long, value_enum)]
    vcs: Option<Vcs>,

    /// The project author.
    #[arg(long)]
    author: Vec<String>,
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

        let vcs = match self.vcs {
            Some(vcs) => vcs,
            None if !is_terminal => Vcs::None,
            None => {
                let selection = Select::new()
                    .with_prompt("Version Control System")
                    .items(Vcs::VARIANTS)
                    .default(0)
                    .interact()?;

                Vcs::VARIANTS[selection]
            }
        };

        let authors = match self.author.is_empty() {
            false => self.author,
            true if !is_terminal => Vec::new(),
            true => {
                let mut author = format!("The {name} Project Developers");

                if let Vcs::Git = vcs {
                    if let Some(git_author) = Git::get_author() {
                        author = git_author;
                    }
                };

                let author = Input::<String>::new()
                    .with_prompt("Author")
                    .with_initial_text(author)
                    .allow_empty(true)
                    .interact_text()?;

                match author.is_empty() {
                    true => Vec::new(),
                    false => vec![author],
                }
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

                if !authors.is_empty() {
                    for author in authors {
                        package.add_author(author);
                    }
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

                if !authors.is_empty() {
                    for author in authors {
                        package.add_author(author);
                    }
                }

                package.add_file("src/lib.rs", b"\n");
                package.add_file("CHANGELOG.md", Changelog::new().to_string().into_bytes());
                project.add_package(package)?;
            }
            Template::None => {}
        }

        if let Vcs::Git = vcs {
            if let Template::CargoBin | Template::CargoLib = template {
                project.add_file(".gitignore", b"/target\n");
            }
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

        let project = project.write(&self.path, false).with_context(|| {
            format!(
                "Could not create project at directory `{}`",
                self.path.display()
            )
        })?;

        if let Vcs::Git = vcs {
            project.init_git()?;
        }

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

#[derive(Clone, Copy, Debug, Display, VariantArray, ValueEnum)]
enum Vcs {
    #[strum(to_string = "Git")]
    Git,
    #[strum(to_string = "None")]
    None,
}
