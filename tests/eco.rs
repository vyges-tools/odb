// SPDX-License-Identifier: Apache-2.0
use vyges_opendb::Db;

const FIXTURE: &str = "tests/fixtures/counter.odb";

#[test]
fn read_basics() {
    let db = Db::open(FIXTURE).expect("open");
    assert_eq!(db.block_name(), "counter");
    assert_eq!(db.num_insts(), 229);
    assert_eq!(db.num_nets(), 52);
    assert_eq!(db.inst_names().len(), 229);
}

// Find an instance with an input pin that is driven by a net.
fn find_driven_input(db: &Db) -> (String, String, String) {
    for i in 0..db.num_insts() {
        let inst = db.nth_inst_name(i);
        let pin = db.input_pin(&inst);
        if pin.is_empty() {
            continue;
        }
        let net = db.net_of(&inst, &pin);
        if !net.is_empty() {
            return (inst, pin, net);
        }
    }
    panic!("no driven input pin found in fixture");
}

#[test]
fn insert_buffer_end_to_end() {
    let mut db = Db::open(FIXTURE).unwrap();
    let (n0, m0) = (db.num_insts(), db.num_nets());

    let buf_master = db.find_master("buf");
    assert!(!buf_master.is_empty(), "no buffer master in fixture libs");
    let (inst, pin, driver) = find_driven_input(&db);

    db.insert_buffer(&inst, &pin, &buf_master, "vyges_buf0", 10_000, 10_000)
        .expect("insert_buffer");

    // one new inst + one new net
    assert_eq!(db.num_insts(), n0 + 1);
    assert_eq!(db.num_nets(), m0 + 1);
    // target pin moved to the new net; buffer input picked up the original driver
    assert_eq!(db.net_of(&inst, &pin), "vyges_buf0_net");
    let a = db.input_pin("vyges_buf0");
    assert_eq!(db.net_of("vyges_buf0", &a), driver);

    // the edit survives serialization
    let out = std::env::temp_dir().join("vyges_opendb_eco.odb");
    db.write(&out).unwrap();
    let db2 = Db::open(&out).unwrap();
    assert_eq!(db2.num_insts(), n0 + 1);
    assert_eq!(db2.net_of(&inst, &pin), "vyges_buf0_net");
}

#[test]
fn insert_eco_buffers_step() {
    use vyges_opendb::eco::{insert_eco_buffers, EcoBuffer};
    let mut db = Db::open(FIXTURE).unwrap();
    let (n0, m0) = (db.num_insts(), db.num_nets());
    let buf = db.find_master("buf");
    let (inst, pin, _driver) = find_driven_input(&db);

    let specs = vec![EcoBuffer { target: format!("{inst}/{pin}"), buffer: buf }];
    let n = insert_eco_buffers(&mut db, &specs).expect("insert_eco_buffers");

    assert_eq!(n, 1);
    assert_eq!(db.num_insts(), n0 + 1);
    assert_eq!(db.num_nets(), m0 + 1);
    assert_eq!(db.net_of(&inst, &pin), "eco_buffer_0_net");
}

#[test]
fn errors_are_typed() {
    let mut db = Db::open(FIXTURE).unwrap();
    assert!(db.create_inst("no_such_master", "x").is_err());
    assert!(db.insert_buffer("no_inst", "A", "no_master", "b", 0, 0).is_err());
}
