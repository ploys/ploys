use std::env;
use std::fmt::{self, Display};
use std::fs;

use anyhow::{anyhow, Error};
use clap::Args;
use console::style;
use gix::remote::Direction;
use gix::Repository;

/// Inspects the project.
#[derive(Args)]
pub struct Inspect;

impl Inspect {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let project = Project::load(gix::open(".")?)?;

        println!("{project}");

        Ok(())
    }
}

/// The project information.
struct Project {
    name: String,
    repo: Option<String>,
}

impl Project {
    /// Loads the project information.
    fn load(repository: Repository) -> Result<Self, Error> {
        Ok(Self {
            name: Self::query_project_name()?,
            repo: Self::query_repository_url(&repository)?,
        })
    }

    /// Queries the project name.
    fn query_project_name() -> Result<String, Error> {
        if let Ok(readme) = fs::read_to_string("README.md") {
            if let Some(title) = readme.lines().find(|line| line.starts_with("# ")) {
                return Ok(title[2..].to_string());
            }
        }

        if let Some(file_stem) = env::current_dir()?.canonicalize()?.file_stem() {
            return Ok(file_stem.to_string_lossy().to_string());
        }

        Err(anyhow!("Invalid project information"))
    }

    /// Queries the repository URL.
    fn query_repository_url(repository: &Repository) -> Result<Option<String>, Error> {
        if let Some(remote) = repository
            .find_default_remote(Direction::Push)
            .transpose()?
        {
            if let Some(url) = remote.url(Direction::Push) {
                return Ok(Some(url.to_bstring().to_string()));
            }
        }

        Ok(None)
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:\n", style("Project").underlined().bold())?;
        writeln!(f, "Name:       {}", self.name)?;
        writeln!(
            f,
            "Repository: {}",
            self.repo.as_deref().unwrap_or_default()
        )?;

        Ok(())
    }
}
