mod parsers;
mod strip_decor;

use anyhow::{bail, ensure, Context};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;
use toml_edit::DocumentMut;

pub(crate) struct Template {
    pub file: File,
    pub path: PathBuf,
    pwd: PathBuf,
    prune_dependencies: bool,
}

impl Template {
    pub fn open(path: PathBuf, prune_dependencies: bool) -> anyhow::Result<Template> {
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

        let template = Template {
            file,
            path,
            pwd,
            prune_dependencies,
        };
        Ok(template)
    }

    pub fn deserialize_manifest(&self) -> anyhow::Result<DocumentMut> {
        let manifest_path = self.locate_manifest()?;
        let mut manifest: DocumentMut = std::fs::read_to_string(manifest_path)?.parse()?;

        strip_decor::strip_decor(&mut manifest);

        if self.prune_dependencies {
            self.prune_unused_deps(&mut manifest)?;
        }

        Ok(manifest)
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

    fn prune_unused_deps(&self, manifest: &mut DocumentMut) -> anyhow::Result<()> {
        // XXX: from here on, if something fails, then I guess we can just not prune anything,
        // instead of bubbling up the error and exiting

        let output = Command::new("cargo")
            .arg("udeps")
            .arg("--output=json")
            .current_dir(&self.pwd)
            .output()?;

        parsers::check_udeps_stderr(output.stderr)?;

        let unused_deps = parsers::parse_udeps_stdout(&output.stdout)
            .context("unexpected output from cargo-udeps")?;

        let mut prune = |deps_set_name: &str, unused_deps: BTreeSet<String>| {
            if let Some(deps_set) = manifest
                .get_mut(deps_set_name)
                .and_then(toml_edit::Item::as_table_mut)
            {
                deps_set.retain(|name, _| !unused_deps.contains(name));
            }
        };

        prune("dependencies", unused_deps.normal);
        prune("dev-dependencies", unused_deps.development);
        prune("build-dependencies", unused_deps.build);

        Ok(())
    }
}
