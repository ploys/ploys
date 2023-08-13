use ploys::project::{Error, Project};

#[test]
#[ignore]
fn test_valid_local_project() -> Result<(), Error> {
    let project = Project::local("../..")?;

    assert_eq!(project.get_name()?, "ploys");
    assert_eq!(
        project.get_url()?,
        "https://github.com/ploys/ploys.git".parse().unwrap()
    );

    Ok(())
}

#[test]
#[ignore]
fn test_valid_remote_project() -> Result<(), Error> {
    let project = Project::remote("ploys/ploys")?;

    assert_eq!(project.get_name()?, "ploys");
    assert_eq!(
        project.get_url()?,
        "https://github.com/ploys/ploys".parse().unwrap()
    );

    Ok(())
}
