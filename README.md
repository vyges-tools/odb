# vyges-odb

A safe, ergonomic Rust API over OpenROAD's OpenDB (`libodb`), built on the low-level
[`vyges-odb-sys`](https://github.com/vyges-tools/odb-sys) FFI — no Tcl, no SWIG, no
OpenROAD engines. Runs on linux/x86_64, linux/arm64, and macOS/Apple Silicon.

> Part of Vyges Loom. This is the layer Loom steps and the ECO applier use — idiomatic Rust
> over the in-memory design database, with the OpenROAD engines driven separately.

## API

```rust
use vyges_odb::Db;

let mut db = Db::open("design.odb")?;
println!("{} — {} insts", db.block_name(), db.num_insts());

// ECO: insert a buffer on a pin (legalization delegated to the engines separately)
let buf = db.find_master("buf");
db.insert_buffer("inst42", "A", &buf, "eco_buf0", 10_000, 10_000)?;

db.write("design_eco.odb")?;
```

- `&self` for reads, `&mut self` for edits — the borrow checker enforces no read-while-mutate.
- Errors are typed (`vyges_odb::Error`) and carry the OpenDB message.
- Write primitives: `create_net`, `create_inst`, `set_inst_location`, `connect`, `disconnect`,
  plus the composed `insert_buffer` — the `InsertECOBuffers` building blocks.

## Build & test

```sh
cargo test
```

The first build compiles a standalone `libodb` via `vyges-odb-sys` (which sparse-checks-out the
pinned OpenROAD subtree and builds it — see that crate for details). Deps: a C++20 compiler +
`cmake boost zlib abseil spdlog fmt`.

## Status

Read + ECO write path over the db core (v0). LEF/DEF/GDS I/O and richer traversal follow the
`vyges-odb-sys` roadmap. OpenROAD is BSD-3-Clause; this crate is Apache-2.0 (see NOTICE).
