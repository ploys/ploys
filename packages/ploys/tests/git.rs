use ploys::project::{Error, Project};
use ploys::repository::revision::Revision;
use ploys::repository::types::git::{CommitParams, Error as GitError, Git};
use ploys::repository::{Commit, Open, Repository, Stage};
use tempfile::tempdir;

#[test]
fn test_repository() -> Result<(), GitError> {
    let dir = tempdir()?;
    let mut repo = Git::init(dir.path())?;

    repo.add_file("hello.txt", "Hello World!")?;
    repo.add_file("foo/bar/baz", "Baz")?;
    repo.commit(CommitParams::new("Initial commit").signed(false))?;

    assert_eq!(repo.get_file("hello.txt")?, Some("Hello World!".into()));
    assert_eq!(repo.get_file("foo/bar/baz")?, Some("Baz".into()));

    repo.add_file("README.md", "# Readme")?;
    repo.remove_file("hello.txt")?;
    repo.remove_file("foo/bar/baz")?;
    repo.commit(CommitParams::new("Second commit").signed(false))?;

    assert_eq!(repo.get_file("README.md")?, Some("# Readme".into()));
    assert_eq!(repo.get_file("hello.txt")?, None);
    assert_eq!(repo.get_file("foo/bar/baz")?, None);

    let mut repo = Git::open(dir.path())?;

    assert_eq!(repo.get_file("README.md")?, Some("# Readme".into()));
    assert_eq!(repo.get_file("hello.txt")?, None);
    assert_eq!(repo.get_file("foo/bar/baz")?, None);

    repo.add_file("Ploys.toml", "[project]\nname = \"example\"")?;
    repo.commit(CommitParams::new("Third commit").signed(false))?;

    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );

    let branch_name = gix::open(dir.path())?
        .head()?
        .referent_name()
        .map(|name| name.shorten().to_string())
        .unwrap_or_else(|| gix::init::DEFAULT_BRANCH_NAME.to_string());

    let mut repo = Git::open(dir.path())?.with_revision(Revision::branch(branch_name));

    repo.add_file("Cargo.toml", "[package]\nname = \"example\"")?;
    repo.commit(CommitParams::new("Fourth commit").signed(false))?;

    assert_eq!(
        repo.get_file("Cargo.toml")?,
        Some("[package]\nname = \"example\"".into())
    );

    let sha = gix::open(dir.path())?
        .head_commit()
        .unwrap()
        .id()
        .to_string();

    let mut repo = Git::open(dir.path())?.with_revision(Revision::sha(sha));

    repo.remove_file("Cargo.toml")?;
    repo.add_file("commit", "5")?;
    repo.commit(CommitParams::new("Fifth commit").signed(false))?;

    assert_eq!(repo.get_file("Cargo.toml")?, None);
    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );
    assert_eq!(repo.get_file("commit")?, Some("5".into()));

    let repo = Git::open(dir.path())?;

    assert_eq!(
        repo.get_file("Cargo.toml")?,
        Some("[package]\nname = \"example\"".into())
    );
    assert_eq!(
        repo.get_file("Ploys.toml")?,
        Some("[project]\nname = \"example\"".into())
    );
    assert_eq!(repo.get_file("commit")?, None);

    dir.close()?;

    Ok(())
}

#[cfg(unix)]
#[test]
fn test_commit_signing() -> Result<(), GitError> {
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir()?;

    Git::init(dir.path())?;

    let git_dir = gix::open(dir.path())?.git_dir().to_owned();
    let signer = git_dir.join("test-signer");
    let config = git_dir.join("config");
    let existing_config = fs::read_to_string(&config)?;

    fs::write(
        &config,
        format!(
            "{existing_config}\n[user]\nname = Test User\nemail = test@example.com\n\
             signingKey = \"{}\"\n[commit]\ngpgSign = false\n[gpg]\nformat = ssh\n\
             [gpg \"ssh\"]\nprogram = \"{}\"\n",
            git_dir.join("test-key.pub").display(),
            signer.display()
        ),
    )?;
    fs::write(&signer, "#!/bin/sh\nexit 1\n")?;
    fs::set_permissions(&signer, fs::Permissions::from_mode(0o700))?;

    let mut repo = Git::open(dir.path())?;

    repo.add_file("hello.txt", "Hello World!")?;

    assert!(
        repo.commit(CommitParams::new("Initial commit").signed(true))
            .is_err()
    );
    assert!(gix::open(dir.path())?.head()?.is_unborn());
    assert_eq!(repo.get_file("hello.txt")?, Some("Hello World!".into()));

    fs::write(
        signer,
        "#!/bin/sh\nset -eu\n\
         test \"$1 $2 $3 $4 $5\" = '-Y sign -n git -f'\n\
         printf '%s\\n' '-----BEGIN SSH SIGNATURE-----' 'fixture' \
         '-----END SSH SIGNATURE-----' > \"$7.sig\"\n",
    )?;
    repo.commit(CommitParams::new("Initial commit").signed(true))?;

    let git = gix::open(dir.path())?;
    let root = git.head_commit()?;

    assert!(root.decode()?.extra_headers().pgp_signature().is_some());
    assert_eq!(repo.get_file("hello.txt")?, Some("Hello World!".into()));

    repo.commit("Unsigned by configuration")?;

    let unsigned = git.head_commit()?;

    assert!(unsigned.decode()?.extra_headers().pgp_signature().is_none());
    assert_eq!(unsigned.parent_ids().next(), Some(root.id()));

    writeln!(
        fs::OpenOptions::new().append(true).open(config)?,
        "\n[commit]\n\tgpgSign = true"
    )?;

    let mut detached = Git::open(dir.path())?.with_revision(Revision::sha(root.id().to_string()));

    detached.commit("Detached signed commit")?;

    let Revision::Sha(sha) = detached.revision() else {
        panic!("expected a detached commit ID");
    };
    let commit = git.rev_parse_single(sha.as_str())?.object()?.into_commit();

    assert!(commit.decode()?.extra_headers().pgp_signature().is_some());
    assert_eq!(git.head_commit()?.id(), unsigned.id());

    let mut repo = Git::open(dir.path())?;

    repo.commit(CommitParams::new("Unsigned commit").signed(false))?;

    let commit = gix::open(dir.path())?;
    let commit = commit.head_commit()?;

    assert!(commit.decode()?.extra_headers().pgp_signature().is_none());
    assert_eq!(commit.parent_ids().next(), Some(unsigned.id()));

    Ok(())
}

#[test]
#[ignore]
fn test_project() -> Result<(), Error<GitError>> {
    let project = Project::git("../..")?;

    assert_eq!(project.name(), "ploys");
    assert_eq!(project.repository().unwrap().full_name(), "ploys/ploys");

    let packages = project.packages().collect::<Vec<_>>();

    let ploys = packages.iter().find(|pkg| pkg.name() == "ploys").unwrap();

    assert_eq!(ploys.name(), "ploys");
    assert_eq!(ploys.path(), "packages/ploys");
    assert_eq!(ploys.manifest_path(), "packages/ploys/Cargo.toml");

    let ploys_api = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-api")
        .unwrap();

    assert_eq!(ploys_api.name(), "ploys-api");
    assert_eq!(ploys_api.path(), "packages/ploys-api");
    assert_eq!(ploys_api.manifest_path(), "packages/ploys-api/Cargo.toml");

    let ploys_cli = packages
        .iter()
        .find(|pkg| pkg.name() == "ploys-cli")
        .unwrap();

    assert_eq!(ploys_cli.name(), "ploys-cli");
    assert_eq!(ploys_cli.path(), "packages/ploys-cli");
    assert_eq!(ploys_cli.manifest_path(), "packages/ploys-cli/Cargo.toml");

    Ok(())
}
