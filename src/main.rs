use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use anyhow::Context;
use clap::{Args, Parser};

fn main() -> anyhow::Result<()> {
    let args = NewScriptCli::parse().args();
    let include_shebang = !args.no_shebang;
    let include_frontmatter = !args.no_frontmatter;

    let mut script = File::create_new(format!("{}.rs", args.script_name))
        .context("Failed to create script file")?;

    match (include_shebang, include_frontmatter) {
        (true, true) => {
            writeln!(script, "{}", shebang(!args.stable, !args.verbose_script))?;
            writeln!(script, "{}\n", frontmatter(args.release))?;
        }
        (true, false) => {
            writeln!(script, "{}\n", shebang(!args.stable, !args.verbose_script))?;
        }
        (false, true) => {
            writeln!(script, "{}\n", frontmatter(args.release))?;
        }
        (false, false) => {}
    }

    writeln!(script, "{}", main_function())?;

    let mut permissions = script.metadata()?.permissions();
    permissions.set_mode(0o755);
    script.set_permissions(permissions)?;

    Ok(())
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

fn frontmatter(release_profile: bool) -> String {
    let mut buf = String::new();
    
    buf.push_str("---\n");
    buf.push_str("[dependencies]\n\n");

    if release_profile {
        buf.push_str("[profile.dev]\n");
        buf.push_str(release_profile_settings());
    }

    buf.push_str("---");

    buf
}

fn release_profile_settings() -> &'static str {
    r#"opt-level = 3
debug = false
debuginfo = "None"
debug-assertions = false
overflow-checks = false
incremental = false
codegen-units = 16
"#
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
    NewScript(NewSciptArgs),
}

#[derive(Debug, Args)]
#[command(version, about, long_about = None)]
struct NewSciptArgs {
    /// Name of the new cargo script without the `.rs` extension
    script_name: String,

    /// Do not include the frontmatter section
    #[arg(long)]
    no_frontmatter: bool,

    /// Do not include the shebang line
    #[arg(long)]
    no_shebang: bool,

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
    fn args(self) -> NewSciptArgs {
        match self {
            NewScriptCli::NewScript(args) => args,
        }
    }
}
