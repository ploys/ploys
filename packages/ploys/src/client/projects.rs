use std::vec::IntoIter;

use serde::Deserialize;
use url::Url;

use crate::project::Project;
use crate::repository::types::github::GitHub;

use super::{Client, Error};

/// The projects iterator.
pub struct Projects<'a> {
    repositories: Repositories<'a>,
}

impl<'a> Projects<'a> {
    /// Constructs a new projects iterator.
    pub fn new(client: &'a Client) -> Self {
        Self {
            repositories: Repositories::new(client),
        }
    }
}

impl Iterator for Projects<'_> {
    type Item = Result<Project<GitHub>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.repositories.next() {
            Ok(Some(repository)) => Some(
                self.repositories
                    .installations
                    .client
                    .get_project(repository.full_name),
            ),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

struct Repositories<'a> {
    installations: Installations<'a>,
    repositories: IntoIter<Repository>,
    next: Option<Url>,
}

impl<'a> Repositories<'a> {
    fn new(client: &'a Client) -> Self {
        Self {
            installations: Installations::new(client),
            repositories: Vec::new().into_iter(),
            next: None,
        }
    }

    fn next(&mut self) -> Result<Option<Repository>, Error> {
        loop {
            if let Some(repository) = self.repositories.next() {
                return Ok(Some(repository));
            }

            let Some(url) = self.next.take() else {
                match self.installations.next()? {
                    Some(installation) => {
                        self.next = Some(
                            Url::parse(&format!(
                                "https://api.github.com/user/installations/{}/repositories",
                                installation.id
                            ))
                            .expect("valid url"),
                        );
                    }
                    None => return Ok(None),
                }

                continue;
            };

            let credentials = self.installations.client.login()?;
            let access_token = credentials.access_token().value();
            let response = self
                .installations
                .client
                .http_client
                .get(url)
                .header("Authorization", format!("Bearer {access_token}"))
                .send()?
                .error_for_status()?;

            self.next = response
                .headers()
                .get("link")
                .and_then(|value| value.to_str().ok())
                .and_then(parse_next_link);

            self.repositories = response
                .json::<RepositoriesResponse>()?
                .repositories
                .into_iter();
        }
    }
}

#[derive(Deserialize)]
struct RepositoriesResponse {
    repositories: Vec<Repository>,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

struct Installations<'a> {
    client: &'a Client,
    installations: IntoIter<Installation>,
    next: Option<Url>,
}

impl<'a> Installations<'a> {
    fn new(client: &'a Client) -> Self {
        Self {
            client,
            installations: Vec::new().into_iter(),
            next: Some(Url::parse("https://api.github.com/user/installations").expect("valid url")),
        }
    }

    fn next(&mut self) -> Result<Option<Installation>, Error> {
        loop {
            if let Some(installation) = self.installations.next() {
                return Ok(Some(installation));
            }

            let Some(url) = self.next.take() else {
                return Ok(None);
            };

            let credentials = self.client.login()?;
            let access_token = credentials.access_token().value();
            let response = self
                .client
                .http_client
                .get(url)
                .header("Authorization", format!("Bearer {access_token}"))
                .send()?
                .error_for_status()?;

            self.next = response
                .headers()
                .get("link")
                .and_then(|value| value.to_str().ok())
                .and_then(parse_next_link);

            self.installations = response
                .json::<InstallationsResponse>()?
                .installations
                .into_iter();
        }
    }
}

#[derive(Deserialize)]
struct InstallationsResponse {
    installations: Vec<Installation>,
}

#[derive(Deserialize)]
struct Installation {
    id: u64,
}

fn parse_next_link(header: &str) -> Option<Url> {
    header
        .split(',')
        .map(str::trim)
        .filter_map(|entry| {
            let mut parts = entry.split(';').map(str::trim);

            let url_part = parts.next()?;
            let rel_part = parts.next()?;

            let url = url_part.strip_prefix('<')?.strip_suffix('>')?;
            let rel = rel_part.strip_prefix("rel=\"")?.strip_suffix('"')?;

            Some((rel, url))
        })
        .find(|(rel, _)| *rel == "next")
        .and_then(|(_, url)| Url::parse(url).ok())
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::parse_next_link;

    #[test]
    fn test_parse_link_header() {
        let header = r#"<https://api.github.com/repos/ploys/ploys/issues?page=2>; rel="prev",
            <https://api.github.com/repos/ploys/ploys/issues?page=4>; rel="next",
            <https://api.github.com/repos/ploys/ploys/issues?page=515>; rel="last",
            <https://api.github.com/repos/ploys/ploys/issues?page=1>; rel="first"#;

        assert_eq!(
            parse_next_link(header),
            Some(Url::parse("https://api.github.com/repos/ploys/ploys/issues?page=4").unwrap())
        );
    }
}
