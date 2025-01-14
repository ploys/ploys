use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

use crate::changelog::{Change, Changeset, Release};

use super::{Error, Repo};

/// Gets the changelog release for the given package version.
pub(super) fn get_release(
    repository: &Repo,
    package: &str,
    version: &Version,
    is_primary: bool,
    token: Option<&str>,
) -> Result<Release, Error> {
    let tags = get_all_tags(repository, token)?;
    let tagname = match is_primary {
        true => version.to_string(),
        false => format!("{package}-{version}"),
    };

    let tag = tags.iter().find(|tag| tag.name == tagname);

    let prev_version = get_previous_version(package, version, is_primary, &tags);
    let prev_tag = prev_version.as_ref().map(|version| match is_primary {
        true => version.to_string(),
        false => format!("{package}-{version}"),
    });

    let timestamp = tag
        .as_ref()
        .map(|tag| tag.target.committed_date)
        .unwrap_or_else(OffsetDateTime::now_utc);

    let pull_requests = match (prev_tag, tag) {
        (None, None) => self::all(repository, token)?,
        (None, Some(tag)) => self::until(repository, &tag.name, &tag.target.oid, token)?,
        (Some(_), _) if prev_version.expect("prev") > *version => Vec::new(),
        (Some(from), None) => self::between(repository, &from, "HEAD", token)?,
        (Some(from), Some(to)) => self::between(repository, &from, &to.name, token)?,
    };

    let package_label = format!("package: {package}");
    let pull_requests = pull_requests
        .into_iter()
        .filter(|pull_request| {
            pull_request
                .labels
                .nodes
                .iter()
                .any(|label| label.name == package_label)
        })
        .filter(|pull_request| {
            !pull_request
                .labels
                .nodes
                .iter()
                .any(|label| label.name.contains("release"))
        });

    let mut release = Release::new(version.to_string());
    let mut changeset = Changeset::changed();

    release.set_date(timestamp.format(&Iso8601::DATE).expect("date"));
    release.set_url(format!(
        "https://github.com/{}/{}/releases/tag/{tagname}",
        repository.owner(),
        repository.name()
    ));

    for pull_request in pull_requests {
        changeset.add_change(
            Change::new(pull_request.title)
                .with_url(format!("#{}", pull_request.number), pull_request.permalink),
        );
    }

    if changeset.changes().count() > 0 {
        release.add_changeset(changeset);
    }

    Ok(release)
}

static ALL_TAGS_QUERY: &str = r#"
query($owner: String!, $name: String!, $cursor: String) {
  repository(owner: $owner, name: $name) {
    refs(refPrefix: "refs/tags/", first: 100, after: $cursor) {
      pageInfo {
        endCursor
        hasNextPage
      }
      nodes {
        name
        target {
          oid
          ... on Commit {
            committedDate
          }
        }
      }
    }
  }
}
"#;

/// Gets all tags.
fn get_all_tags(repository: &Repo, token: Option<&str>) -> Result<Vec<GitTag>, Error> {
    let mut tags = Vec::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .json(&Query {
                query: ALL_TAGS_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: None,
                    to: None,
                    cursor: cursor.as_deref(),
                },
            })
            .send()?
            .error_for_status()?
            .json::<MatchingTagsResponse>()?;

        tags.extend(response.data.repository.refs.nodes);

        if response.data.repository.refs.page_info.has_next_page {
            cursor = response.data.repository.refs.page_info.end_cursor;

            continue;
        }

        break;
    }

    Ok(tags)
}

/// Gets the previous version or last version if no previous exists.
fn get_previous_version(
    package: &str,
    version: &Version,
    is_primary: bool,
    tags: &[GitTag],
) -> Option<Version> {
    let mut versions = tags
        .iter()
        .filter_map(|tag| match is_primary {
            true => tag.name.parse::<Version>().ok(),
            false => {
                match tag.name.starts_with(package)
                    && tag.name.as_bytes().get(package.len()) == Some(&b'-')
                {
                    true => tag.name[package.len() + 1..].parse::<Version>().ok(),
                    false => None,
                }
            }
        })
        .collect::<Vec<_>>();

    versions.sort();

    let previous_version = versions
        .iter()
        .filter(|previous_version| *previous_version < version)
        .filter(|previous_version| version.pre.is_empty() || previous_version.pre.is_empty())
        .last();

    match previous_version {
        Some(previous_version) => Some(previous_version.clone()),
        None => match versions.last() {
            Some(last_version) if last_version != version => Some(last_version.clone()),
            _ => None,
        },
    }
}

static ALL_QUERY: &str = r#"
query($owner: String!, $name: String!, $cursor: String) {
    repository(owner: $owner, name: $name) {
        defaultBranchRef {
            target {
                ... on Commit {
                    history(first: 100, after: $cursor) {
                        pageInfo {
                            endCursor
                            hasNextPage
                        }
                        nodes {
                            oid
                            associatedPullRequests(first: 1) {
                                nodes {
                                    number
                                    title
                                    mergedAt
                                    permalink
                                    labels(first: 20) {
                                        nodes {
                                            name
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;

/// Gets all pull requests.
fn all(repository: &Repo, token: Option<&str>) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .json(&Query {
                query: ALL_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: None,
                    to: None,
                    cursor: cursor.as_deref(),
                },
            })
            .send()?
            .error_for_status()?
            .json::<AllResponse>()?;

        let commits = response.data.repository.default_branch_ref.target.history;

        for commit in commits.nodes {
            for pull_request in commit.associated_pull_requests.nodes {
                pull_requests
                    .entry(pull_request.number)
                    .or_insert(pull_request);
            }
        }

        if commits.page_info.has_next_page {
            cursor = commits.page_info.end_cursor;

            continue;
        }

        break;
    }

    let mut pull_requests = pull_requests.into_values().collect::<Vec<_>>();

    pull_requests.sort_by_key(|pull_request| pull_request.merged_at);

    Ok(pull_requests)
}

static UNTIL_QUERY: &str = r#"
query($owner: String!, $name: String!, $to: String!, $cursor: String) {
    repository(owner: $owner, name: $name) {
        ref(qualifiedName: $to) {
            target {
                ... on Commit {
                    history(first: 100, after: $cursor) {
                        pageInfo {
                            endCursor
                            hasNextPage
                        }
                        nodes {
                            oid
                            associatedPullRequests(first: 1) {
                                nodes {
                                    number
                                    title
                                    mergedAt
                                    permalink
                                    labels(first: 20) {
                                        nodes {
                                            name
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;

/// Gets pull requests until the specified ref.
fn until(
    repository: &Repo,
    to: &str,
    sha: &str,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .json(&Query {
                query: UNTIL_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: None,
                    to: Some(to),
                    cursor: cursor.as_deref(),
                },
            })
            .send()?
            .error_for_status()?
            .json::<UntilResponse>()?;

        let commits = response.data.repository.r#ref.target.history;

        for commit in commits.nodes {
            if commit.oid == sha {
                continue;
            }

            for pull_request in commit.associated_pull_requests.nodes {
                pull_requests
                    .entry(pull_request.number)
                    .or_insert(pull_request);
            }
        }

        if commits.page_info.has_next_page {
            cursor = commits.page_info.end_cursor;

            continue;
        }

        break;
    }

    let mut pull_requests = pull_requests.into_values().collect::<Vec<_>>();

    pull_requests.sort_by_key(|pull_request| pull_request.merged_at);

    Ok(pull_requests)
}

static BETWEEN_QUERY: &str = r#"
query($owner: String!, $name: String!, $from: String!, $to: String!, $cursor: String) {
    repository(owner: $owner, name: $name) {
        ref(qualifiedName: $from) {
            compare(headRef: $to) {
                commits(first: 100, after: $cursor) {
                    pageInfo {
                        endCursor
                        hasNextPage
                    }
                    nodes {
                        oid
                        associatedPullRequests(first: 1) {
                            nodes {
                                number
                                title
                                mergedAt
                                permalink
                                labels(first: 20) {
                                    nodes {
                                        name
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;

/// Gets pull requests between two refs.
fn between(
    repository: &Repo,
    from: &str,
    to: &str,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .json(&Query {
                query: BETWEEN_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: Some(from),
                    to: Some(to),
                    cursor: cursor.as_deref(),
                },
            })
            .send()?
            .error_for_status()?
            .json::<BetweenResponse>()?;

        let commits = response.data.repository.r#ref.compare.commits;

        for commit in commits.nodes {
            for pull_request in commit.associated_pull_requests.nodes {
                pull_requests
                    .entry(pull_request.number)
                    .or_insert(pull_request);
            }
        }

        if commits.page_info.has_next_page {
            cursor = commits.page_info.end_cursor;

            continue;
        }

        break;
    }

    let mut pull_requests = pull_requests.into_values().collect::<Vec<_>>();

    pull_requests.sort_by_key(|pull_request| pull_request.merged_at);

    Ok(pull_requests)
}

#[derive(Serialize)]
struct Variables<'a> {
    owner: &'a str,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    to: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cursor: Option<&'a str>,
}

#[derive(Serialize)]
struct Query<'a> {
    query: &'a str,
    variables: Variables<'a>,
}

#[derive(Deserialize)]
struct MatchingTagsResponse {
    data: MatchingTagsResponseData,
}

#[derive(Deserialize)]
struct AllResponse {
    data: AllResponseData,
}

#[derive(Deserialize)]
struct UntilResponse {
    data: UntilResponseData,
}

#[derive(Deserialize)]
struct BetweenResponse {
    data: BetweenResponseData,
}

#[derive(Deserialize)]
struct MatchingTagsResponseData {
    repository: MatchingTagsResponseRepository,
}

#[derive(Deserialize)]
struct AllResponseData {
    repository: AllResponseRepository,
}

#[derive(Deserialize)]
struct UntilResponseData {
    repository: UntilResponseRepository,
}

#[derive(Deserialize)]
struct BetweenResponseData {
    repository: BetweenResponseRepository,
}

#[derive(Deserialize)]
struct MatchingTagsResponseRepository {
    refs: MatchingTagsResponseRefs,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AllResponseRepository {
    default_branch_ref: AllResponseRef,
}

#[derive(Deserialize)]
struct UntilResponseRepository {
    r#ref: UntilResponseRef,
}

#[derive(Deserialize)]
struct BetweenResponseRepository {
    r#ref: BetweenResponseRef,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MatchingTagsResponseRefs {
    page_info: ResponsePageInfo,
    nodes: Vec<GitTag>,
}

#[derive(Deserialize)]
struct AllResponseRef {
    target: AllResponseTarget,
}

#[derive(Deserialize)]
struct UntilResponseRef {
    target: UntilResponseTarget,
}

#[derive(Deserialize)]
struct BetweenResponseRef {
    compare: BetweenResponseCompare,
}

#[derive(Deserialize)]
struct AllResponseTarget {
    history: ResponseCommits,
}

#[derive(Deserialize)]
struct UntilResponseTarget {
    history: ResponseCommits,
}

#[derive(Deserialize)]
struct BetweenResponseCompare {
    commits: ResponseCommits,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseCommits {
    page_info: ResponsePageInfo,
    nodes: Vec<Commit>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponsePageInfo {
    end_cursor: Option<String>,
    has_next_page: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    oid: String,
    associated_pull_requests: PullRequests,
}

#[derive(Deserialize)]
struct PullRequests {
    nodes: Vec<PullRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    number: u64,
    title: String,
    #[serde(with = "time::serde::iso8601")]
    merged_at: OffsetDateTime,
    permalink: String,
    labels: Labels,
}

#[derive(Deserialize)]
struct Labels {
    nodes: Vec<Label>,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

#[derive(Deserialize)]
struct GitTag {
    name: String,
    target: GitObject,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GitObject {
    oid: String,
    #[serde(with = "time::serde::iso8601")]
    committed_date: OffsetDateTime,
}
