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
