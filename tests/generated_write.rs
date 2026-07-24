// SPDX-License-Identifier: Apache-2.0
// The generated setter surface (L2/write) — only compiled/run under `--features gen-write`.
// Proves the three setter param paths (scalar, enum, and error-on-missing) round-trip against
// the generated read accessors.
#![cfg(feature = "gen-write")]

use vyges_opendb::Db;

const FIXTURE: &str = "tests/fixtures/counter.odb";

#[test]
fn generated_scalar_setter_round_trips() {
    let mut db = Db::open(FIXTURE).unwrap();
    let net = db.net_names().into_iter().next().unwrap();
    db.net_set_weight(&net, 42).unwrap();
    assert_eq!(db.net_get_weight(&net), 42);

    // the edit survives serialization
    let out = std::env::temp_dir().join("vyges_opendb_gen_write.odb");
    db.write(&out).unwrap();
    assert_eq!(Db::open(&out).unwrap().net_get_weight(&net), 42);
}

#[test]
fn generated_enum_setter_round_trips() {
    let mut db = Db::open(FIXTURE).unwrap();
    let inst = db.nth_inst_name(0);
    // dbOrientType parses "MX"; the generated enum-param setter constructs it from the string
    db.inst_set_orient(&inst, "MX").unwrap();
    assert_eq!(db.inst_get_orient(&inst), "MX");
}

#[test]
fn generated_setter_errs_on_missing_object() {
    let mut db = Db::open(FIXTURE).unwrap();
    // addressing a non-existent net must surface a typed error, not a panic or silent no-op
    assert!(db.net_set_weight("no_such_net", 1).is_err());
}
