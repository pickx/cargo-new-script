use anyhow::{bail, Context};
use serde::Deserialize;
use std::collections::BTreeSet;
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
    pub root: String,
}

pub fn parse_udeps_stdout(stdout: &[u8]) -> Option<OutcomeUnusedDeps> {
    let json = serde_json::from_slice::<serde_json::Value>(stdout).ok()?;

    // when this is `true`, `unused_deps` is going to be an empty object,
    // so there's no point in continuing here
    let success = json.get("success")?.as_bool()?;
    if success {
        let empty = OutcomeUnusedDeps::default();
        return Some(empty);
    }

    // XXX: a fragile kludge. I don't really know what I'm doing.
    let outcome = json
        .get("unused_deps")?
        .as_object()?
        .values()
        .next()?
        .to_owned();

    serde_json::from_value(outcome).ok()
}

/// some shallow attempts at detecting errors
pub fn check_udeps_stderr(stderr: Vec<u8>) -> anyhow::Result<()> {
    let preamble = "failed to run cargo-udeps";

    let stderr =
        String::from_utf8(stderr).with_context(|| format!("{preamble} - malformed stderr"))?;

    if stderr.starts_with("error: no such command") {
        bail!("{preamble} - is cargo-udeps installed?")
    } else if stderr.starts_with("error: failed to write") {
        bail!("{preamble} - permission error")
    } else if stderr.contains("nightly compiler") {
        bail!("{preamble} - are you using a nightly toolchain?")
    }

    Ok(())
}

/// simplified from <https://github.com/est31/cargo-udeps/blob/44e6e220ba90ff81d0777aeef45b7b8022dd120a/src/lib.rs#L1041>
#[derive(Deserialize, Default)]
pub struct OutcomeUnusedDeps {
    pub normal: BTreeSet<String>,
    pub development: BTreeSet<String>,
    pub build: BTreeSet<String>,
}
