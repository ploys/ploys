use std::str::FromStr;

use anyhow::{anyhow, Error};
use clap::Args;
use console::style;
use ploys::project::source::github::{GitHub, Repository};
use ploys::project::source::Source;
use ploys::project::Project;
use url::Url;

/// Gets the project information.
#[derive(Args)]
pub struct Info {
    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoOrUrl>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Info {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match &self.remote {
            Some(remote) => match &self.token {
                Some(token) => self.print(Project::<GitHub>::github_with_authentication_token(
                    remote.clone().try_into_repo()?.to_string(),
                    token,
                )?),
                None => self.print(Project::github(
                    remote.clone().try_into_repo()?.to_string(),
                )?),
            },
            None => self.print(Project::git(".")?),
        }
    }

    pub fn print<T>(&self, project: Project<T>) -> Result<(), Error>
    where
        T: Source,
        ploys::project::Error: From<T::Error>,
    {
        println!("{}:\n", style("Project").underlined().bold());
        println!("Name:       {}", project.name());
        println!("Repository: {}", project.get_url()?);

        println!("\n{}:\n", style("Packages").underlined().bold());

        let packages = project.packages();
        let max_name_len = packages
            .iter()
            .map(|pkg| pkg.name().len())
            .max()
            .unwrap_or_default();
        let max_version_len = packages
            .iter()
            .map(|pkg| pkg.version().len())
            .max()
            .unwrap_or_default();

        for package in packages {
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

#[derive(Clone)]
enum RepoOrUrl {
    Repo(Repository),
    Url(Url),
}

impl RepoOrUrl {
    fn try_into_repo(self) -> Result<Repository, Error> {
        self.try_into()
    }
}

impl FromStr for RepoOrUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Repository>() {
            Ok(repo) => Ok(Self::Repo(repo)),
            Err(_) => match s.parse::<Url>() {
                Ok(url) => Ok(Self::Url(url)),
                Err(_) => Err(anyhow!("Expected owner/repo or URL, found: {}", s)),
            },
        }
    }
}

impl TryFrom<RepoOrUrl> for Repository {
    type Error = Error;

    fn try_from(value: RepoOrUrl) -> Result<Self, Self::Error> {
        match value {
            RepoOrUrl::Repo(repo) => Ok(repo),
            RepoOrUrl::Url(url) => {
                if url.domain() != Some("github.com") {
                    return Err(anyhow!(
                        "Unsupported remote repository: Only GitHub is supported"
                    ));
                }

                Ok(url
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches(".git")
                    .parse::<Repository>()?)
            }
        }
    }
}
