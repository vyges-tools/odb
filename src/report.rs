// SPDX-License-Identifier: Apache-2.0
//! Read-only audit/report steps over the design database — Loom-native equivalents of LibreLane's
//! read-only `Odb.*` reporting steps. These never mutate the database; output is structured (JSON).

use serde::Serialize;
use std::collections::HashMap;

use crate::Db;

/// One row of a cell-frequency table.
#[derive(Debug, Clone, Serialize)]
pub struct CellFreq {
    pub master: String,
    pub count: usize,
}

/// `CellFrequencyTables`: count instances per master cell, most-used first (ties by name).
/// Mirrors LibreLane's `Odb.CellFrequencyTables`.
pub fn cell_frequency_table(db: &Db) -> Vec<CellFreq> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for inst in db.inst_names() {
        let m = db.inst_master(&inst);
        if !m.is_empty() {
            *counts.entry(m).or_default() += 1;
        }
    }
    let mut rows: Vec<CellFreq> =
        counts.into_iter().map(|(master, count)| CellFreq { master, count }).collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.master.cmp(&b.master)));
    rows
}

/// `ReportDisconnectedPins`: every instance pin (`inst/pin`) and port (`port:name`) with no net.
/// Mirrors LibreLane's `Odb.ReportDisconnectedPins`.
pub fn disconnected_pins(db: &Db) -> Vec<String> {
    let mut out = Vec::new();
    for inst in db.inst_names() {
        for pin in db.iterm_names(&inst) {
            if db.net_of(&inst, &pin).is_empty() {
                out.push(format!("{inst}/{pin}"));
            }
        }
    }
    for port in db.bterm_names() {
        if db.bterm_net(&port).is_empty() {
            out.push(format!("port:{port}"));
        }
    }
    out
}

/// One net's connectivity: its type, special flag, and the pins it touches.
#[derive(Debug, Clone, Serialize)]
pub struct NetConn {
    pub net: String,
    pub sig_type: String,
    pub special: bool,
    /// Instance pins (`inst/pin`) on the net.
    pub iterms: Vec<String>,
    /// Block ports on the net.
    pub bterms: Vec<String>,
    /// Total pin count (fanout+1): `iterms + bterms`.
    pub degree: usize,
}

/// Connectivity graph: one `NetConn` per net (its sig-type, special flag, and the pins it touches),
/// highest-degree net first. This is the core instrumentation primitive — a netlist connectivity
/// dump that higher layers turn into fanout histograms, high-fanout-net reports, clock/power-net
/// audits, etc. Read-only; no LibreLane counterpart (it's an odb-native traversal).
pub fn net_connectivity(db: &Db) -> Vec<NetConn> {
    let mut rows: Vec<NetConn> = db
        .net_names()
        .into_iter()
        .map(|net| {
            let iterms = db.net_iterms(&net);
            let bterms = db.net_bterms(&net);
            let degree = iterms.len() + bterms.len();
            NetConn {
                sig_type: db.net_sigtype(&net),
                special: db.net_is_special(&net),
                iterms,
                bterms,
                degree,
                net,
            }
        })
        .collect();
    rows.sort_by(|a, b| b.degree.cmp(&a.degree).then_with(|| a.net.cmp(&b.net)));
    rows
}

/// `WriteVerilogHeader`: a Verilog module header (`module <name>(...); input/output ...`) built
/// from the block's ports + directions. Mirrors LibreLane's `Odb.WriteVerilogHeader` (header only —
/// no cell instantiations). Returns the Verilog text.
pub fn verilog_header(db: &Db) -> String {
    let ports = db.bterm_names();
    let mut v = format!("module {} (\n", db.block_name());
    for (i, p) in ports.iter().enumerate() {
        let comma = if i + 1 < ports.len() { "," } else { "" };
        v.push_str(&format!("  {p}{comma}\n"));
    }
    v.push_str(");\n");
    for p in &ports {
        let dir = match db.bterm_direction(p).as_str() {
            "INPUT" => "input",
            "OUTPUT" => "output",
            _ => "inout",
        };
        v.push_str(&format!("  {dir} {p};\n"));
    }
    v.push_str("endmodule\n");
    v
}

