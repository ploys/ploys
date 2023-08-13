use std::str::FromStr;

use anyhow::{anyhow, Error};
use clap::Args;
use console::style;
use ploys::project::remote::Repository;
use ploys::project::Project;
use url::Url;

/// Inspects the project.
#[derive(Args)]
pub struct Inspect {
    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoOrUrl>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Inspect {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let project = match self.remote {
            Some(remote) => match self.token {
                Some(token) => Project::remote_with_authentication_token(
                    remote.try_into_repo()?.to_string(),
                    token,
                )?,
                None => Project::remote(remote.try_into_repo()?.to_string())?,
            },
            None => Project::local(".")?,
        };

        println!("{}:\n", style("Project").underlined().bold());
        println!("Name:       {}", project.get_name()?);
        println!("Repository: {}", project.get_url()?);

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
