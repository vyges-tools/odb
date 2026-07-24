// SPDX-License-Identifier: Apache-2.0
//! `vyges-opendb` — OpenROAD's OpenDB (libodb) design-database CLI, shipped by `vyges install opendb`.
//!
//! A thin multi-tool over the safe [`vyges_opendb`] API (OpenROAD's OpenDB / libodb). Unix-only:
//! libodb is native C++ and is not built on non-unix targets.
//!
//! Subcommands:
//!   info                 read a `.odb` and print a one-line block summary (read path).
//!   insert-eco-buffers   splice ECO buffers into a `.odb` (Loom step; LibreLane-compatible
//!                        `Odb.InsertECOBuffers` database surgery). Legalization is separate.
//!
//! Arg parsing is deliberately hand-rolled (no clap) to match the rest of the suite and keep
//! the dependency surface minimal.
use serde::Deserialize;
use vyges_opendb::{eco, Db};

type Fail = Box<dyn std::error::Error>;

const USAGE: &str = "\
vyges-opendb — OpenROAD's OpenDB (libodb) design database

usage:
  vyges-opendb <command> [options]

commands:
  info                --input <f.odb>
                      Print a one-line summary: block name + inst/net/bterm counts.

  insert-eco-buffers  --input <in.odb> --output <out.odb> [--config <eco.json>]
                      Insert ECO buffers (INSERT_ECO_BUFFERS in the config) into the design.

  --version, -V       Print the version.
  --help,    -h       Print this help.
";

fn main() {
    if let Err(e) = run() {
        eprintln!("vyges-opendb: error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Fail> {
    let mut args = std::env::args().skip(1);
    let cmd = args.next().unwrap_or_default();
    match cmd.as_str() {
        "info" => info(args),
        "insert-eco-buffers" => insert_eco_buffers(args),
        "-V" | "--version" => {
            println!("vyges-opendb {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "" | "-h" | "--help" => {
            print!("{USAGE}");
            Ok(())
        }
        other => Err(format!("unknown command '{other}'. Try 'vyges-opendb --help'.").into()),
    }
}

/// `info --input <f.odb>` — read a design and print a one-line summary.
fn info(mut args: impl Iterator<Item = String>) -> Result<(), Fail> {
    let mut input = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--input" | "-i" => input = args.next(),
            "-h" | "--help" => {
                eprintln!("usage: vyges-opendb info --input <f.odb>");
                return Ok(());
            }
            other => return Err(format!("info: unknown argument: {other}").into()),
        }
    }
    let input = input.ok_or("info: --input <f.odb> required")?;
    let db = Db::open(&input)?;
    println!(
        "{input}: block={} insts={} nets={} bterms={}",
        db.block_name(),
        db.num_insts(),
        db.num_nets(),
        db.num_bterms(),
    );
    Ok(())
}

#[derive(Deserialize, Default)]
struct EcoConfig {
    #[serde(rename = "INSERT_ECO_BUFFERS", default)]
    insert_eco_buffers: Vec<eco::EcoBuffer>,
}

/// Machine-readable step contract (the Vyges/Loom step convention): identity, the CLI args, and
/// the config schema — so an orchestrator (Sley / Loom auto-mode) can introspect a step without
/// running it. `insert-eco-buffers --describe` emits this; every step ships the same shape.
const INSERT_ECO_BUFFERS_DESCRIBE: &str = r#"{
  "step": "insert-eco-buffers",
  "summary": "Splice ECO buffers into a placed .odb (database surgery; legalization is a separate step).",
  "librelane_equivalent": "Odb.InsertECOBuffers",
  "unix_only": true,
  "args": [
    { "name": "--input",  "kind": "input",  "type": "path", "required": true,  "description": "input .odb design" },
    { "name": "--output", "kind": "output", "type": "path", "required": true,  "description": "output .odb after ECO" },
    { "name": "--config", "kind": "config", "type": "path", "required": false, "description": "JSON with INSERT_ECO_BUFFERS (default: no-op)" }
  ],
  "config_schema": {
    "INSERT_ECO_BUFFERS": {
      "type": "array",
      "description": "buffers to insert; each rewires the target pin's driver through a new buffer",
      "item": {
        "target": { "type": "string", "description": "instance/pin to buffer, e.g. inst42/A" },
        "buffer": { "type": "string", "description": "library cell master, e.g. sky130_fd_sc_hd__buf_2" }
      }
    }
  }
}"#;

/// `insert-eco-buffers --input <in.odb> --output <out.odb> [--config <eco.json>] | --describe`.
fn insert_eco_buffers(mut args: impl Iterator<Item = String>) -> Result<(), Fail> {
    let (mut input, mut output, mut config) = (None, None, None);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--input" | "-i" => input = args.next(),
            "--output" | "-o" => output = args.next(),
            "--config" | "-c" => config = args.next(),
            "--describe" => {
                println!("{INSERT_ECO_BUFFERS_DESCRIBE}");
                return Ok(());
            }
            "-h" | "--help" => {
                eprintln!("usage: vyges-opendb insert-eco-buffers --input <in.odb> --output <out.odb> --config <eco.json>");
                eprintln!("       vyges-opendb insert-eco-buffers --describe   # JSON step contract");
                return Ok(());
            }
            other => return Err(format!("insert-eco-buffers: unknown argument: {other}").into()),
        }
    }
    let input = input.ok_or("insert-eco-buffers: --input <in.odb> required")?;
    let output = output.ok_or("insert-eco-buffers: --output <out.odb> required")?;
    let cfg: EcoConfig = match config {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(&p)?)?,
        None => EcoConfig::default(),
    };

    let mut db = Db::open(&input)?;
    let n = eco::insert_eco_buffers(&mut db, &cfg.insert_eco_buffers)?;
    db.write(&output)?;
    eprintln!("insert-eco-buffers: inserted {n} buffer(s), {input} -> {output}");
    Ok(())
}
