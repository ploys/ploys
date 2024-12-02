use anyhow::Error;
use clap::Args;
use console::style;
use ploys::project::Project;
use ploys::repository::revision::Revision;

use crate::util::repo_or_url::RepoOrUrl;

/// Gets the project information.
#[derive(Args)]
pub struct Info {
    /// The remote GitHub repository owner/repo or URL.
    #[clap(long)]
    remote: Option<RepoOrUrl>,

    /// The target branch name.
    #[arg(long, conflicts_with_all = ["tag", "sha"])]
    branch: Option<String>,

    /// The target tag name.
    #[arg(long, conflicts_with_all = ["branch", "sha"])]
    tag: Option<String>,

    /// The target commit SHA.
    #[arg(long, conflicts_with_all = ["branch", "tag"])]
    sha: Option<String>,

    /// The authentication token for GitHub API access.
    #[clap(long, env = "GITHUB_TOKEN")]
    token: Option<String>,
}

impl Info {
    /// Executes the command.
    pub fn exec(self) -> Result<(), Error> {
        match &self.remote {
            Some(remote) => match &self.token {
                Some(token) => self.print(Project::github_with_revision_and_authentication_token(
                    remote.clone().try_into_repo()?.to_string(),
                    self.revision(),
                    token,
                )?),
                None => self.print(Project::github_with_revision(
                    remote.clone().try_into_repo()?.to_string(),
                    self.revision(),
                )?),
            },
            None => self.print(Project::git_with_revision(".", self.revision())?),
        }
    }

    /// Gets the Git revision.
    pub fn revision(&self) -> Revision {
        match &self.branch {
            Some(branch) => Revision::branch(branch),
            None => match &self.tag {
                Some(tag) => Revision::tag(tag),
                None => match &self.sha {
                    Some(sha) => Revision::sha(sha),
                    None => Revision::head(),
                },
            },
        }
    }

    pub fn print(&self, project: Project) -> Result<(), Error> {
        println!("{}:\n", style("Project").underlined().bold());
        println!("Name:       {}", project.name());
        println!("Repository: {}", project.get_url()?);

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
