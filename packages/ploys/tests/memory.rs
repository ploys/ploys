use ploys::project::Project;
use ploys::repository::memory::Memory;
use tempfile::tempdir;

#[test]
fn test_project_write() -> Result<(), Box<dyn std::error::Error>> {
    let memory = Memory::new()
        .with_file("Ploys.toml", b"[project]\nname = \"example\"")
        .with_file("Cargo.toml", b"[workspace]\nmembers = [\"packages/*\"]")
        .with_file(
            "packages/example/Cargo.toml",
            b"[package]\nname = \"example\"",
        );

    let dir = tempdir()?;
    let project = Project::open(memory)?;
    let project = project.write(dir.path(), false)?;
    let package = project.get_package("example").unwrap();

    assert_eq!(package.name(), "example");

    dir.close()?;

    Ok(())
}
