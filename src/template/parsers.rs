use serde::Deserialize;
use std::path::PathBuf;

pub fn parse_locate_project_stdout(stdout: &[u8]) -> Option<PathBuf> {
    let json: serde_json::Value = serde_json::from_slice(stdout).ok()?;

    let project_location: ProjectLocation = serde_json::from_value(json).ok()?;
    let root_path = PathBuf::from(project_location.root);
    Some(root_path)
}

/// simplified from <https://github.com/rust-lang/cargo/blob/852a31615d8ad66cf9768e16ef50119806629027/src/bin/cargo/commands/locate_project.rs#L24>
#[derive(Deserialize)]
struct ProjectLocation {
    root: String,
}
