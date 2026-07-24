// SPDX-License-Identifier: Apache-2.0
//! ECO steps over the design database — Loom-native equivalents of LibreLane's `Odb.*`
//! surgery steps. These mutate the database only; legalization (incremental routing /
//! detailed placement) is delegated to the OpenROAD engines as separate flow steps.

use serde::Deserialize;

use crate::{Db, Error, Result};

/// One entry of the `INSERT_ECO_BUFFERS` config — the LibreLane-compatible shape.
#[derive(Debug, Clone, Deserialize)]
pub struct EcoBuffer {
    /// `"instance/pin"` — the pin to buffer.
    pub target: String,
    /// Buffer master cell name.
    pub buffer: String,
}

/// Apply `InsertECOBuffers`: for each spec, splice its buffer onto the target pin, placing
/// the buffer at the target instance's location. Returns the number of buffers inserted.
///
/// Mirrors LibreLane's `Odb.InsertECOBuffers` (`eco_buffer.py`) database surgery — the
/// downstream `grt` incremental-route + `dpl` legalization is a separate engine step.
pub fn insert_eco_buffers(db: &mut Db, specs: &[EcoBuffer]) -> Result<usize> {
    for (i, spec) in specs.iter().enumerate() {
        let (inst, pin) = spec.target.split_once('/').ok_or_else(|| {
            Error::Odb(format!("bad target '{}' (expected inst/pin)", spec.target))
        })?;
        let (x, y) = db.inst_location(inst);
        let name = format!("eco_buffer_{i}");
        db.insert_buffer(inst, pin, &spec.buffer, &name, x, y)?;
    }
    Ok(specs.len())
}

/// One entry of the `INSERT_ECO_DIODES` config — the LibreLane-compatible shape.
#[derive(Debug, Clone, Deserialize)]
pub struct EcoDiode {
    /// `"instance/pin"` — the pin whose net gets an antenna diode.
    pub target: String,
    /// Antenna-diode master cell name.
    pub diode: String,
}

/// Apply `InsertECODiodes`: for each spec, tie its antenna diode onto the target pin's net,
/// placing the diode at the target instance's location. Returns the number of diodes inserted.
///
/// Mirrors LibreLane's `Odb.InsertECODiodes` database surgery — a diode is a leaf tied onto an
/// existing net (no rewiring, unlike a buffer). Downstream legalization is a separate engine step.
pub fn insert_eco_diodes(db: &mut Db, specs: &[EcoDiode]) -> Result<usize> {
    for (i, spec) in specs.iter().enumerate() {
        let (inst, pin) = spec.target.split_once('/').ok_or_else(|| {
            Error::Odb(format!("bad target '{}' (expected inst/pin)", spec.target))
        })?;
        let (x, y) = db.inst_location(inst);
        let name = format!("eco_diode_{i}");
        db.insert_diode(inst, pin, &spec.diode, &name, x, y)?;
    }
    Ok(specs.len())
}

/// One entry of the `MANUAL_GLOBAL_PLACEMENT` config: an instance and its origin (DBU).
#[derive(Debug, Clone, Deserialize)]
pub struct GlobalPlacement {
    pub instance: String,
    pub x: i32,
    pub y: i32,
}

/// Apply `ManualGlobalPlacement`: set each listed instance's origin. Returns the count placed.
/// Mirrors LibreLane's `Odb.ManualGlobalPlacement` — fixes specific cells before global placement.
pub fn manual_global_placement(db: &mut Db, specs: &[GlobalPlacement]) -> Result<usize> {
    for spec in specs {
        db.set_inst_location(&spec.instance, spec.x, spec.y)?;
    }
    Ok(specs.len())
}

/// One entry of the `MANUAL_MACRO_PLACEMENT` config: instance, origin (DBU), and orientation.
#[derive(Debug, Clone, Deserialize)]
pub struct MacroPlacement {
    pub instance: String,
    pub x: i32,
    pub y: i32,
    /// `R0`/`R90`/`R180`/`R270`/`MX`/`MY`/`MXR90`/`MYR90`; omitted leaves the orientation as-is.
    #[serde(default)]
    pub orient: Option<String>,
}

/// Apply `ManualMacroPlacement`: place each macro at its origin + orientation. Returns the count.
/// Mirrors LibreLane's `Odb.ManualMacroPlacement` (macros are placed + oriented before the flow).
pub fn manual_macro_placement(db: &mut Db, specs: &[MacroPlacement]) -> Result<usize> {
    for spec in specs {
        db.set_inst_location(&spec.instance, spec.x, spec.y)?;
        if let Some(orient) = &spec.orient {
            db.set_inst_orient(&spec.instance, orient)?;
        }
    }
    Ok(specs.len())
}

/// The `DIODES_ON_PORTS` config: the diode cell, and optionally specific ports (default: all).
#[derive(Debug, Clone, Deserialize)]
pub struct DiodesOnPorts {
    /// Antenna-diode master cell name.
    pub diode: String,
    /// Specific port (bterm) names; empty = every port that carries a net.
    #[serde(default)]
    pub ports: Vec<String>,
}

/// Apply `DiodesOnPorts`: tie an antenna diode onto each selected port's net, placed at the
/// port's first-pin location. Returns the number of diodes inserted (unconnected ports skipped).
/// Mirrors LibreLane's `Odb.DiodesOnPorts` — blanket antenna protection on I/O ports.
pub fn diodes_on_ports(db: &mut Db, spec: &DiodesOnPorts) -> Result<usize> {
    let ports = if spec.ports.is_empty() {
        db.bterm_names()
    } else {
        spec.ports.clone()
    };
    let mut n = 0;
    for (i, port) in ports.iter().enumerate() {
        let net = db.bterm_net(port);
        if net.is_empty() {
            continue; // unconnected port — nothing to protect
        }
        let (x, y) = db.bterm_location(port);
        db.insert_diode_on_net(&net, &spec.diode, &format!("port_diode_{i}"), x, y)?;
        n += 1;
    }
    Ok(n)
}
