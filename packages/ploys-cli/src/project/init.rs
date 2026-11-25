use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Context, Error, bail};
use clap::Args;
use dialoguer::Input;
use ploys::project::Project;
use ploys::repository::RepoAddr;
use ploys::repository::types::git::Git;

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
    repository: Option<RepoAddr>,

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

        let authors = match self.author.is_empty() {
            false => self.author,
            true if !is_terminal => Vec::new(),
            true => {
                let author = match Git::get_author() {
                    Some(author) => author,
                    None => format!("The {name} Project Developers"),
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

        let mut project = Project::new(name);

        if let Some(description) = description {
            project.set_description(description);
        }

        if !authors.is_empty() {
            project.set_authors(authors);
        }

        if let Some(repository) = repository {
            project.set_repository(repository);
        }

        std::fs::create_dir_all(&self.path)
            .with_context(|| format!("Could not create directory `{}`", self.path.display()))?;

        let first_entry = self
            .path
            .read_dir()
            .and_then(|mut entries| entries.next().transpose())
            .with_context(|| format!("Could not read directory `{}`", self.path.display()))?;

        if first_entry.is_some() {
            bail!("Expected an empty directory at `{}`", self.path.display());
        }

        Git::init(&self.path).with_context(|| {
            format!(
                "Could not initialize Git repository at `{}`",
                self.path.display()
            )
        })?;

        project.write(&self.path, false).with_context(|| {
            format!(
                "Could not create project at directory `{}`",
                self.path.display()
            )
        })?;

        Ok(())
    }
}
