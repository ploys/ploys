use std::env;
use std::fmt::{self, Display};
use std::fs;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use clap::Args;
use console::style;
use gix::remote::Direction;
use gix::Repository;
use url::Url;

/// Inspects the project.
#[derive(Args)]
pub struct Inspect {
    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoOrUrl>,
}

impl Inspect {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        let project = match self.remote {
            Some(remote) => Project::remote(remote)?,
            None => Project::local(gix::open(".")?)?,
        };

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
    /// Loads the project information from the local repository.
    fn local(repository: Repository) -> Result<Self, Error> {
        Ok(Self {
            name: Self::query_local_project_name()?,
            repo: Self::query_repository_url(&repository)?,
        })
    }

    /// Loads the project information from the remote repository.
    fn remote(remote: RepoOrUrl) -> Result<Self, Error> {
        let repo = match remote {
            RepoOrUrl::Repo(repo) => repo,
            RepoOrUrl::Url(url) => {
                if url.domain() != Some("github.com") {
                    return Err(anyhow!(
                        "Unsupported remote repository: Only GitHub is supported"
                    ));
                }

                url.path()
                    .trim_start_matches("/")
                    .trim_end_matches(".git")
                    .parse::<Repo>()?
            }
        };

        Self::query_remote_repository(&repo)?;

        Ok(Self {
            name: Self::query_remote_project_name(&repo),
            repo: Some(format!("https://github.com/{}", repo)),
        })
    }

    /// Queries the local project name.
    fn query_local_project_name() -> Result<String, Error> {
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

    /// Queries the remote repository to check that it exists.
    fn query_remote_repository(repo: &Repo) -> Result<(), Error> {
        let url = format!("https://api.github.com/repos/{}", repo);
        let request = ureq::head(&url).set("User-Agent", "ploys/ploys");

        match request.call() {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(404, _)) => Err(anyhow!("Repository not found")),
            Err(err) => Err(err.into()),
        }
    }

    /// Queries the remote repository name.
    fn query_remote_project_name(repo: &Repo) -> String {
        let url = format!("https://api.github.com/repos/{}/readme", repo);
        let request = ureq::get(&url)
            .set("User-Agent", "ploys/ploys")
            .set("Accept", "application/vnd.github.raw");

        if let Ok(response) = request.call() {
            if let Ok(readme) = response.into_string() {
                if let Some(title) = readme.lines().find(|line| line.starts_with("# ")) {
                    return title[2..].to_string();
                }
            }
        }

        repo.repo.clone()
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

#[derive(Clone)]
struct Repo {
    owner: String,
    repo: String,
}

impl Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)?;

        Ok(())
    }
}

impl FromStr for Repo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("/") {
            Some((owner, repo)) => match repo.contains("/") {
                true => Err(anyhow!("Expected owner/repo, found: {}", s)),
                false => Ok(Self {
                    owner: owner.to_string(),
                    repo: repo.to_string(),
                }),
            },
            None => Err(anyhow!("Expected owner/repo, found: {}", s)),
        }
    }
}

#[derive(Clone)]
enum RepoOrUrl {
    Repo(Repo),
    Url(Url),
}

impl FromStr for RepoOrUrl {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<Repo>() {
            Ok(repo) => Ok(Self::Repo(repo)),
            Err(_) => match s.parse::<Url>() {
                Ok(url) => Ok(Self::Url(url)),
                Err(_) => Err(anyhow!("Expected owner/repo or URL, found: {}", s)),
            },
        }
    }
}
