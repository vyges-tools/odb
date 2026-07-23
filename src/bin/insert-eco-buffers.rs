// SPDX-License-Identifier: Apache-2.0
//! `insert-eco-buffers` — a Loom step. Reads a `.odb` + an `INSERT_ECO_BUFFERS` config,
//! splices the buffers, writes the `.odb`. LibreLane-compatible database surgery
//! (`Odb.InsertECOBuffers`); the downstream grt/dpl legalization is a separate step.
use serde::Deserialize;
use vyges_odb::{eco, Db};

#[derive(Deserialize, Default)]
struct Config {
    #[serde(rename = "INSERT_ECO_BUFFERS", default)]
    insert_eco_buffers: Vec<eco::EcoBuffer>,
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (mut input, mut output, mut config) = (None, None, None);
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--input" | "-i" => input = args.next(),
            "--output" | "-o" => output = args.next(),
            "--config" | "-c" => config = args.next(),
            "-h" | "--help" => {
                eprintln!("usage: insert-eco-buffers --input <in.odb> --output <out.odb> --config <eco.json>");
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    let input = input.ok_or("--input <in.odb> required")?;
    let output = output.ok_or("--output <out.odb> required")?;
    let cfg: Config = match config {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(&p)?)?,
        None => Config::default(),
    };

    let mut db = Db::open(&input)?;
    let n = eco::insert_eco_buffers(&mut db, &cfg.insert_eco_buffers)?;
    db.write(&output)?;
    eprintln!("insert-eco-buffers: inserted {n} buffer(s), {input} -> {output}");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("insert-eco-buffers: error: {e}");
        std::process::exit(1);
    }
}
