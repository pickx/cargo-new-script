use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use anyhow::Context;
use clap::{Args, Parser};

fn main() -> anyhow::Result<()> {
    let args = NewScriptCli::parse().args();

    let mut script = File::create_new(format!("{}.rs", args.script_name))
        .context("Failed to create script file")?;

    if !args.no_shebang {
        writeln!(script, "{}", shebang(!args.stable))?;
    }

    if !args.no_frontmatter {
        writeln!(script, "{}\n", frontmatter())?;
    }

    writeln!(script, "{}", main_function())?;

    let mut permissions = script.metadata()?.permissions();
    permissions.set_mode(0o755);
    script.set_permissions(permissions)?;

    Ok(())
}

fn shebang(nightly: bool) -> &'static str {
    if nightly {
        "#!/usr/bin/env -S cargo +nightly -Zscript"
    } else {
        "#!/usr/bin/env cargo"
    }
}

fn frontmatter() -> &'static str {
    "---
[dependencies]

---"
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
    #[arg(long)]
    stable: bool,
}

impl NewScriptCli {
    fn args(self) -> NewSciptArgs {
        match self {
            NewScriptCli::NewScript(args) => args,
        }
    }
}
