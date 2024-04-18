mod parsers;

use anyhow::{bail, ensure, Context};
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use toml::Table;

pub(crate) struct Template {
    pub file: File,
    pub path: PathBuf,
    pwd: PathBuf,
}

impl Template {
    pub fn open(path: PathBuf) -> anyhow::Result<Template> {
        // NOTE: technically there's no reason why the file must have this extension,
        // but since our workflow involves looking for a manifest file,
        // then in practice that's what it's going to have.
        // this also saves us from opening the file unnecesarilly
        ensure!(
            path.extension().and_then(OsStr::to_str) == Some("rs"),
            "template must have .rs file extension"
        );

        let file = File::open(&path).context("template file does not exist")?;
        let metadata = file.metadata()?;
        ensure!(!metadata.is_dir(), "cannot read template: is a directory");

        let pwd = path
            .parent()
            .map(ToOwned::to_owned)
            .context("no parent directory for template script")?;

        let template = Template { file, path, pwd };
        Ok(template)
    }

    pub fn deserialize_manifest(&self) -> anyhow::Result<Table> {
        let manifest_path = self.locate_manifest()?;
        let manifest = std::fs::read_to_string(manifest_path)?;
        toml::from_str(&manifest).context("could not deserialize project manifest")
    }

    fn locate_manifest(&self) -> anyhow::Result<PathBuf> {
        let output = Command::new("cargo")
            .arg("locate-project")
            .current_dir(&self.pwd)
            .output()
            .context("failed to run cargo locate-manifest")?;

        if !output.stderr.is_empty() {
            // could use a better failure heuristic here
            let path_display = self.path.display();
            bail!("cargo locate-manifest could not find a manifest file for {path_display}")
        }

        parsers::parse_locate_project_stdout(&output.stdout)
            .context("unexpected output from cargo locate-manifest")
    }
}
