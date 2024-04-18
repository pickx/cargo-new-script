mod template;

use anyhow::bail;
use clap::{Args, Parser};
use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use template::Template;
use toml::Table;

fn main() -> anyhow::Result<()> {
    let args = NewScriptCli::parse().args();
    let include_shebang = !args.no_shebang;
    let include_frontmatter = !args.no_frontmatter;

    let mut script = String::new();

    // validates template early on, to avoid unneccessary work
    let template = if let Some(path) = args.template {
        let template = Template::open(path)?;
        Some(template)
    } else {
        None
    };

    if include_shebang {
        let shebang = shebang(!args.stable, !args.verbose_script);
        writeln!(script, "{shebang}")?;
    }

    if include_frontmatter {
        let template_manifest = template
            .as_ref()
            .map(Template::deserialize_manifest)
            .transpose()?;

        let frontmatter_toml = frontmatter_toml(args.release, template_manifest);
        writeln!(script, "---")?;
        write!(script, "{frontmatter_toml}")?; // the `toml` Display impl already terminates with a newline
        writeln!(script, "---")?;
    }

    if let Some(mut template) = template {
        template.file.read_to_string(&mut script)?;
    } else {
        let main = main_function();
        writeln!(script, "{main}")?;
    }

    write_script_to_file(&args.script_name, &script, args.overwrite)
}

fn write_script_to_file(script_name: &str, contents: &str, overwrite: bool) -> anyhow::Result<()> {
    let mut path_to_new_file = PathBuf::from(script_name);

    // accept names that end with `.rs` and just set the extension regardless
    path_to_new_file.set_extension("rs");

    let mut file = open_options(overwrite)
        .open(&path_to_new_file)
        .map_err(|e| {
            let context = if e.kind() == ErrorKind::AlreadyExists {
                "failed to create script file - file already exists. consider using `--overwrite`"
            } else {
                "failed to create script file"
            };

            anyhow::Error::new(e).context(context)
        })?;

    let write_result = std::io::Write::write_all(&mut file, contents.as_bytes());
    if write_result.is_err() {
        // try and clean up, since file is probably fubar
        // since we're gonna bail immediately after this, let's just ignore the error here
        let _ = std::fs::remove_file(path_to_new_file);
        bail!("error while writing to newly-created script");
    }

    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions)?;

    Ok(())
}

fn open_options(overwrite: bool) -> OpenOptions {
    let mut options = File::options();
    let handle = &mut options;

    handle.read(true).write(true);
    if overwrite {
        handle.truncate(true).create(true)
    } else {
        handle.create_new(true)
    };

    options
}

fn shebang(nightly: bool, quiet: bool) -> String {
    let cargo_invocation = if nightly {
        "-S cargo +nightly -Zscript"
    } else {
        "cargo"
    };
    let quiet_arg = if quiet { " --quiet" } else { "" };

    format!("#!/usr/bin/env {cargo_invocation}{quiet_arg}")
}

/// currently, we only use the template manifest's `dependencies`.
fn frontmatter_toml(release_profile: bool, template_manifest: Option<Table>) -> Table {
    let mut root = Table::new();

    let package = Table::from_iter([("edition".into(), "2021".into())]);
    root.insert("package".into(), package.into());

    // if dependencies can be found, use that. otherwise output an empty dependencies header
    //
    // TODO: given a dependency with an inline table as value, such as "clap = { version = "4.5.0", features = ["derive"] }"
    // `toml` will (by design) not preserve the formatting and will serialize it as `[dependencies.clap]`
    // that's still valid and will compile just fine. if we can care we can use `toml_edit`.
    let deps = template_manifest
        .and_then(|manifest| manifest.get("dependencies")?.as_table().cloned())
        .unwrap_or_default();
    root.insert("dependencies".into(), deps.into());

    if release_profile {
        let dev = Table::from_iter([
            ("opt-level".into(), 3.into()),
            ("debug".into(), false.into()),
            ("debug-assertions".into(), false.into()),
            ("overflow-checks".into(), false.into()),
            ("incremental".into(), false.into()),
            ("codegen-units".into(), 16.into()),
        ]);
        let mut profile = Table::new();
        profile.insert("dev".into(), dev.into());
        root.insert("profile".into(), profile.into());
    }

    root
}

fn main_function() -> &'static str {
    "fn main() {
    println!(\"Hello, world!\");
}"
}

#[derive(Debug, Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum NewScriptCli {
    NewScript(NewScriptArgs),
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
struct NewScriptArgs {
    /// Name of the new cargo script, with or without the `.rs` extension
    script_name: String,

    /// Do not include the frontmatter section
    #[arg(long)]
    no_frontmatter: bool,

    /// Do not include the shebang line
    #[arg(long)]
    no_shebang: bool,

    /// Overwrite target file if it already exists
    #[arg(long)]
    overwrite: bool,

    /// Converts an existing (single-file) Rust program to a `cargo-script`, copying its source code and dependencies
    #[arg(long, short, value_name = "PATH", conflicts_with("no_frontmatter"))]
    template: Option<PathBuf>,

    /// Create a shebang line that uses the stable toolchain. Currently, this does not generate a runnable script because `cargo script` requires nightly.
    #[arg(long, conflicts_with("no_shebang"))]
    stable: bool,

    /// Do not add `--quiet` to shebang line. `cargo` log messages will not be suppressed when the script is executed.
    #[arg(long, conflicts_with("no_shebang"))]
    verbose_script: bool,

    /// Generate default `release` profile settings, simulating the optimizations of release mode.
    #[arg(long, conflicts_with("no_frontmatter"))]
    release: bool,
}

impl NewScriptCli {
    fn args(self) -> NewScriptArgs {
        match self {
            NewScriptCli::NewScript(args) => args,
        }
    }
}
