// SPDX-License-Identifier: Apache-2.0
//! `vyges-opendb` — a safe, ergonomic Rust API over OpenROAD's OpenDB (`libodb`).
//!
//! Wraps the low-level [`vyges_opendb_lib`] FFI: an owned [`Db`] handle, `&self` for reads and
//! `&mut self` for edits (so Rust's borrow checker enforces no read-while-mutate aliasing),
//! and typed [`Error`]s from the C++ layer. Objects are addressed by name.
//!
//! The write primitives + [`Db::insert_buffer`] are the building blocks for the ECO applier
//! (`InsertECOBuffers`). Legalization (incremental routing / detailed placement) is delegated
//! to the OpenROAD engines separately — this layer only mutates the database.

// The libodb-backed surface (`Db`, `eco`) is unix-only — libodb is not built on non-unix
// targets. `Error`/`Result` stay cross-platform. See vyges-opendb-lib for the rationale.
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use cxx::UniquePtr;
#[cfg(unix)]
use vyges_opendb_lib as sys;

#[cfg(unix)]
pub mod eco;

/// Errors from the OpenDB layer or path handling.
#[derive(Debug)]
pub enum Error {
    /// An error surfaced by the C++ OpenDB layer.
    Odb(String),
    /// A path that is not valid UTF-8.
    NonUtf8Path,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Odb(m) => write!(f, "{m}"),
            Error::NonUtf8Path => write!(f, "path is not valid UTF-8"),
        }
    }
}
impl std::error::Error for Error {}
impl From<cxx::Exception> for Error {
    fn from(e: cxx::Exception) -> Self {
        Error::Odb(e.what().to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(unix)]
fn path_str(p: impl AsRef<Path>) -> Result<String> {
    p.as_ref().to_str().map(str::to_owned).ok_or(Error::NonUtf8Path)
}

/// An OpenDB design database (owns a `dbDatabase` + its logger). Unix-only.
#[cfg(unix)]
pub struct Db {
    inner: UniquePtr<sys::OdbDb>,
}

#[cfg(unix)]
impl Db {
    /// Read a `.odb` file.
    pub fn open(path: impl AsRef<Path>) -> Result<Db> {
        let inner = sys::open_db(&path_str(path)?)?;
        Ok(Db { inner })
    }

    /// Serialize the database to a `.odb` file.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<()> {
        Ok(sys::write_db(self.r(), &path_str(path)?)?)
    }

    fn r(&self) -> &sys::OdbDb {
        self.inner.as_ref().expect("vyges-opendb: null db handle")
    }

    // ---- read / inspect ------------------------------------------------------
    pub fn block_name(&self) -> String { sys::block_name(self.r()) }
    pub fn num_insts(&self) -> usize { sys::num_insts(self.r()) }
    pub fn num_nets(&self) -> usize { sys::num_nets(self.r()) }
    pub fn num_bterms(&self) -> usize { sys::num_bterms(self.r()) }

    /// Name of the `i`-th instance (empty if out of range).
    pub fn nth_inst_name(&self, i: usize) -> String { sys::nth_inst_name(self.r(), i) }
    /// All instance names.
    pub fn inst_names(&self) -> Vec<String> {
        (0..self.num_insts()).map(|i| self.nth_inst_name(i)).collect()
    }
    /// First library master whose name contains `substr` (empty if none).
    pub fn find_master(&self, substr: &str) -> String { sys::find_master(self.r(), substr) }
    /// First input-signal pin name of `inst` (empty if none).
    pub fn input_pin(&self, inst: &str) -> String { sys::input_pin(self.r(), inst) }
    /// First output-signal pin name of `inst` (empty if none).
    pub fn output_pin(&self, inst: &str) -> String { sys::output_pin(self.r(), inst) }
    /// Net connected to `inst/pin` (empty if unconnected).
    pub fn net_of(&self, inst: &str, pin: &str) -> String { sys::net_of(self.r(), inst, pin) }
    /// Instance origin `(x, y)` in DBU (`(0, 0)` if not found).
    pub fn inst_location(&self, inst: &str) -> (i32, i32) {
        (sys::inst_x(self.r(), inst), sys::inst_y(self.r(), inst))
    }

    // ---- write primitives ----------------------------------------------------
    pub fn create_net(&mut self, name: &str) -> Result<()> {
        Ok(sys::create_net(self.r(), name)?)
    }
    pub fn create_inst(&mut self, master: &str, name: &str) -> Result<()> {
        Ok(sys::create_inst(self.r(), master, name)?)
    }
    pub fn set_inst_location(&mut self, inst: &str, x: i32, y: i32) -> Result<()> {
        Ok(sys::set_inst_location(self.r(), inst, x, y)?)
    }
    pub fn connect(&mut self, inst: &str, pin: &str, net: &str) -> Result<()> {
        Ok(sys::connect(self.r(), inst, pin, net)?)
    }
    pub fn disconnect(&mut self, inst: &str, pin: &str) -> Result<()> {
        Ok(sys::disconnect(self.r(), inst, pin)?)
    }

    // ---- composed ECO op -----------------------------------------------------
    /// Insert `buffer_master` (named `buf_name`, placed at `x,y`) on `target_inst/target_pin`.
    ///
    /// The pin's current driver net now feeds the buffer input; the buffer output drives a
    /// fresh net (`{buf_name}_net`) that the target pin is moved onto. Legalization is a
    /// separate, engine-delegated step.
    pub fn insert_buffer(
        &mut self,
        target_inst: &str,
        target_pin: &str,
        buffer_master: &str,
        buf_name: &str,
        x: i32,
        y: i32,
    ) -> Result<()> {
        let driver = self.net_of(target_inst, target_pin);
        if driver.is_empty() {
            return Err(Error::Odb(format!("no net on {target_inst}/{target_pin}")));
        }
        let new_net = format!("{buf_name}_net");
        self.create_net(&new_net)?;
        self.create_inst(buffer_master, buf_name)?;
        self.set_inst_location(buf_name, x, y)?;

        let a = self.input_pin(buf_name);
        let z = self.output_pin(buf_name);
        if a.is_empty() || z.is_empty() {
            return Err(Error::Odb(format!("{buffer_master} lacks an input or output pin")));
        }
        self.connect(buf_name, &a, &driver)?; // buffer input  <- original driver net
        self.connect(buf_name, &z, &new_net)?; // buffer output -> new net
        self.disconnect(target_inst, target_pin)?; // target pin off the original net
        self.connect(target_inst, target_pin, &new_net)?; // target pin -> new net
        Ok(())
    }
}
