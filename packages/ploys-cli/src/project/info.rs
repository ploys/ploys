use std::path::PathBuf;

use anyhow::{Error, bail};
use clap::Args;
use console::style;
use ploys::project::Project;
use ploys::repository::revision::Revision;
use ploys::repository::{RepoSpec, Repository};

/// Gets the project information.
#[derive(Args)]
pub struct Info {
    /// The target path.
    #[arg(default_value = ".", conflicts_with_all = ["remote"])]
    path: PathBuf,

    /// The remote repository specification.
    #[arg(long, conflicts_with_all = ["path"])]
    remote: Option<RepoSpec>,

    /// The repository head.
    #[arg(long)]
    head: bool,

    /// The target branch name.
    #[arg(long, conflicts_with_all = ["head", "tag", "sha"])]
    branch: Option<String>,

    /// The target tag name.
    #[arg(long, conflicts_with_all = ["head", "branch", "sha"])]
    tag: Option<String>,

    /// The target commit SHA.
    #[arg(long, conflicts_with_all = ["head", "branch", "tag"])]
    sha: Option<String>,

    /// The authentication token for GitHub API access.
    #[arg(long, env = "GITHUB_TOKEN", hide_env_values = true)]
    token: Option<String>,
}

impl Info {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match &self.remote {
            Some(remote) => {
                let Some(github) = remote.to_github() else {
                    bail!("Unsupported remote repository: {remote}");
                };

                let revision = self.revision().unwrap_or_else(Revision::head);
                let project = match &self.token {
                    Some(token) => Project::github_with_revision_and_authentication_token(
                        github, revision, token,
                    )?,
                    None => Project::github_with_revision(github, revision)?,
                };

                self.print(project)?;

                Ok(())
            }
            None => {
                if let Some(revision) = self.revision() {
                    let project = Project::git_with_revision(&self.path, revision)?;

                    self.print(project)?;
                } else {
                    let project = Project::fs(&self.path)?;

                    self.print(project)?;
                }

                Ok(())
            }
        }
    }

    /// Gets the Git revision.
    pub fn revision(&self) -> Option<Revision> {
        match self.head {
            true => Some(Revision::head()),
            false => match &self.branch {
                Some(branch) => Some(Revision::branch(branch)),
                None => match &self.tag {
                    Some(tag) => Some(Revision::tag(tag)),
                    None => self.sha.as_ref().map(Revision::sha),
                },
            },
        }
    }

    pub fn print<T>(&self, project: Project<T>) -> Result<(), Error>
    where
        T: Repository,
    {
        println!("{}:\n", style("Project").underlined().bold());
        println!("Name:        {}", project.name());

        if let Some(description) = project.description() {
            println!("Description: {description}");
        }

        if let Some(repository) = project.repository() {
            println!("Repository:  {}", repository.to_url());
        }

        println!("\n{}:\n", style("Packages").underlined().bold());

        let packages = project.packages().collect::<Vec<_>>();

        let max_name_len = packages
            .iter()
            .map(|pkg| pkg.name().len())
            .max()
            .unwrap_or_default();
        let max_version_len = packages
            .iter()
            .map(|pkg| pkg.version().to_string().len())
            .max()
            .unwrap_or_default();

        for package in packages.iter() {
            println!(
                "{:<max_name_len$}  {:>max_version_len$}  {}",
                package.name(),
                package.version(),
                package.description().unwrap_or_default()
            );
        }

        Ok(())
    }
}
