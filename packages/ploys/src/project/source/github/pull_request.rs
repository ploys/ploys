use std::collections::BTreeMap;

use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::{Error, Repository};

/// Gets the pull requests for the specified version.
pub fn get_pull_requests(
    repository: &Repository,
    package: &str,
    version: &Version,
    is_primary: bool,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, Error> {
    let tag = match is_primary {
        true => version.to_string(),
        false => format!("{package}-{version}"),
    };

    let sha = get_matching_tags(repository, &tag, token)?
        .into_iter()
        .find_map(|git_ref| {
            (git_ref.r#ref.starts_with("refs/tags/") && tag == git_ref.r#ref[10..])
                .then_some(git_ref.object.sha)
        });

    let prev_version = get_previous_version(repository, package, version, is_primary, token)?;
    let prev_tag = prev_version.as_ref().map(|version| match is_primary {
        true => version.to_string(),
        false => format!("{package}-{version}"),
    });

    if let Some(prev_version) = &prev_version {
        if prev_version > version {
            return Ok(Vec::new());
        }
    }

    match (prev_tag, sha) {
        (None, None) => self::all(repository, token),
        (None, Some(sha)) => self::until(repository, &tag, &sha, token),
        (Some(previous), None) => self::between(repository, &previous, "HEAD", token),
        (Some(previous), Some(_)) => self::between(repository, &previous, &tag, token),
    }
}

/// Gets matching tags.
fn get_matching_tags(
    repository: &Repository,
    tag: &str,
    token: Option<&str>,
) -> Result<Vec<GitRef>, Error> {
    Ok(repository
        .get(format!("git/matching-refs/tags/{tag}"), token)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call()?
        .into_json()?)
}

/// Gets the previous version or last version if no previous exists.
fn get_previous_version(
    repository: &Repository,
    package: &str,
    version: &Version,
    is_primary: bool,
    token: Option<&str>,
) -> Result<Option<Version>, Error> {
    let tag = match is_primary {
        true => String::new(),
        false => format!("{package}-"),
    };

    let mut versions = get_matching_tags(repository, &tag, token)?
        .iter()
        .filter_map(|git_ref| match git_ref.r#ref.starts_with("refs/tags/") {
            true => Some(&git_ref.r#ref[10..]),
            false => None,
        })
        .filter_map(|tag| match is_primary {
            true => tag.parse::<Version>().ok(),
            false => {
                match tag.starts_with(package) && tag.as_bytes().get(package.len()) == Some(&b'-') {
                    true => tag[package.len() + 1..].parse::<Version>().ok(),
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

    Ok(match previous_version {
        Some(previous_version) => Some(previous_version.clone()),
        None => match versions.last() {
            Some(last_version) if last_version != version => Some(last_version.clone()),
            _ => None,
        },
    })
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
fn all(repository: &Repository, token: Option<&str>) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .send_json(Query {
                query: ALL_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: None,
                    to: None,
                    cursor: cursor.as_deref(),
                },
            })?
            .into_json::<AllResponse>()?;

        let commits = response.data.repository.default_branch_ref.target.history;

        for commit in commits.nodes {
            for pull_request in commit.associated_pull_requests.nodes {
                pull_requests
                    .entry(pull_request.number)
                    .or_insert(pull_request);
            }
        }

        if commits.page_info.has_next_page {
            cursor = Some(commits.page_info.end_cursor);

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
    repository: &Repository,
    to: &str,
    sha: &str,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .send_json(Query {
                query: UNTIL_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: None,
                    to: Some(to),
                    cursor: cursor.as_deref(),
                },
            })?
            .into_json::<UntilResponse>()?;

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
            cursor = Some(commits.page_info.end_cursor);

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
    repository: &Repository,
    from: &str,
    to: &str,
    token: Option<&str>,
) -> Result<Vec<PullRequest>, Error> {
    let mut pull_requests = BTreeMap::<_, PullRequest>::new();
    let mut cursor = None;

    loop {
        let response = repository
            .graphql(token)
            .send_json(Query {
                query: BETWEEN_QUERY,
                variables: Variables {
                    owner: repository.owner(),
                    name: repository.name(),
                    from: Some(from),
                    to: Some(to),
                    cursor: cursor.as_deref(),
                },
            })?
            .into_json::<BetweenResponse>()?;

        let commits = response.data.repository.r#ref.compare.commits;

        for commit in commits.nodes {
            for pull_request in commit.associated_pull_requests.nodes {
                pull_requests
                    .entry(pull_request.number)
                    .or_insert(pull_request);
            }
        }

        if commits.page_info.has_next_page {
            cursor = Some(commits.page_info.end_cursor);

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
    end_cursor: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    #[serde(with = "time::serde::iso8601")]
    pub merged_at: OffsetDateTime,
    pub permalink: String,
    pub labels: Labels,
}

#[derive(Debug, Deserialize)]
pub struct Labels {
    pub nodes: Vec<Label>,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub name: String,
}

#[derive(Deserialize)]
struct GitRef {
    r#ref: String,
    object: GitObject,
}

#[derive(Deserialize)]
struct GitObject {
    sha: String,
}
