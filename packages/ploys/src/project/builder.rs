use serde::Serialize;

use crate::client::{Client, Credentials, Error as ClientError};
use crate::project::Config;
use crate::repository::revision::Revision;
use crate::repository::types::github::GitHub;
use crate::repository::{GitLike, RepoAddr};

use super::{Error as ProjError, Project};

/// The project builder.
pub struct Builder {
    repo: RepoAddr,
    client: Client,
    description: Option<String>,
    authors: Vec<String>,
    private: bool,
}

impl Builder {
    /// Constructs a new project builder.
    pub(crate) fn new(repo: RepoAddr, client: Client) -> Self {
        Self {
            repo,
            client,
            description: None,
            authors: Vec::new(),
            private: false,
        }
    }
}

impl Builder {
    /// Builds the project with the given description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builds the project with the given authors.
    pub fn with_authors(mut self, authors: impl IntoIterator<Item = String>) -> Self {
        self.authors = authors.into_iter().collect();
        self
    }

    /// Builds the project with private visibility.
    pub fn with_private_visibility(mut self) -> Self {
        self.private = true;
        self
    }

    /// Finishes creating the project.
    pub fn finished(self) -> Result<Project<GitHub>, ClientError> {
        let credentials = self.client.login()?;
        let (repo, branch) = self.init_repo(&credentials)?;
        let config = self.init_config();

        self.commit_config(&repo, &branch, config)?;

        Ok(Project::open(repo)?)
    }
}

impl Builder {
    /// Initialises the repository.
    ///
    /// This passes the `auto_init` parameter to initialise the repository with
    /// an initial commit containing a basic `README.md`. This is necessary to
    /// ensure that the commit is signed by GitHub when using a GitHub App user
    /// access token.
    ///
    /// The only alternative would be using the App installation to listen to
    /// repository creation and create a commit. However, that would not work
    /// with a personal access token because the app would not be installed on
    /// the repository. It would also be limited in the configuration options.
    ///
    /// The `createCommitOnBranch` mutation does not work on an empty repo, the
    /// file contents endpoint doesn't sign the commit and is limited to a
    /// single file per commit, and the git commits endpoint only signs when
    /// using an App installation access token.
    fn init_repo(&self, credentials: &Credentials) -> Result<(GitHub, String), ClientError> {
        let url = match self.repo.owner() == credentials.user() {
            true => String::from("https://api.github.com/user/repos"),
            false => format!("https://api.github.com/orgs/{}/repos", self.repo.owner()),
        };

        self.client
            .http_client()
            .post(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2026-03-10")
            .bearer_auth(credentials.access_token().value())
            .json(&RepoParams {
                name: self.repo.name().to_owned(),
                description: self.description.clone(),
                private: self.private,
                auto_init: true,
                has_issues: true,
                has_projects: true,
                allow_squash_merge: true,
                allow_merge_commit: false,
                allow_rebase_merge: false,
                delete_branch_on_merge: true,
                squash_merge_commit_title: SquashMergeCommitTitle::Pr,
            })
            .send()?
            .error_for_status()?;

        let mut repo =
            GitHub::new(self.client.clone(), self.repo.clone()).map_err(ProjError::Repository)?;

        let branch = repo.get_default_branch().map_err(ProjError::Repository)?;

        repo.set_revision(Revision::branch(&branch));

        Ok((repo, branch))
    }

    /// Creates the project configuration.
    fn init_config(&self) -> Config {
        let mut config = Config::new(self.repo.name());

        if let Some(description) = &self.description {
            config.set_project_description(description);
        }

        if !self.authors.is_empty() {
            config.set_project_authors(self.authors.clone());
        }

        config.set_project_repository(self.repo.clone());
        config
    }

    /// Commits the project configuration.
    fn commit_config(
        &self,
        repo: &GitHub,
        branch: &str,
        config: Config,
    ) -> Result<(), ClientError> {
        let files = vec![("Ploys.toml".into(), config.to_string())];

        repo.commit_branch(branch, "Add project configuration", files)
            .map_err(ProjError::Repository)?;

        Ok(())
    }
}

#[derive(Serialize)]
struct RepoParams {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    private: bool,
    auto_init: bool,
    has_issues: bool,
    has_projects: bool,
    allow_squash_merge: bool,
    allow_merge_commit: bool,
    allow_rebase_merge: bool,
    delete_branch_on_merge: bool,
    squash_merge_commit_title: SquashMergeCommitTitle,
}

#[derive(Serialize)]
enum SquashMergeCommitTitle {
    #[serde(rename = "PR_TITLE")]
    Pr,
}
