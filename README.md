# cargo-new-script

`cargo-new-script` is a `cargo` command to quickly generate a cargo-script.

As of February 2024, cargo-script is available on nightly. See the tracking issues for [`cargo`](https://github.com/rust-lang/cargo/issues/12207) and [`rustc`](https://github.com/rust-lang/rfcs/pull/3503).

## Installation

```shell
cargo install --git https://github.com/avsaase/cargo-new-script
```

Currently, only unix operating systems are supported.

## Usage

```shell
cargo new-script my-script
```

This generates:

```rust
#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]

---

fn main() {
    println!("Hello, world!");
}
```

Run your script with `./my-script.rs`. The generated script is already made executable.

See `cargo new-script --help` for limited options.
