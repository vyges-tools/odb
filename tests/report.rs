// SPDX-License-Identifier: Apache-2.0
use vyges_opendb::{report, Db};

const FIXTURE: &str = "tests/fixtures/counter.odb";

#[test]
fn cell_frequency_table_counts_all_masters() {
    let db = Db::open(FIXTURE).unwrap();
    let table = report::cell_frequency_table(&db);
    assert!(!table.is_empty());
    // every instance has a master, so the counts sum to the instance count
    let total: usize = table.iter().map(|r| r.count).sum();
    assert_eq!(total, db.num_insts());
    // most-used first
    for w in table.windows(2) {
        assert!(w[0].count >= w[1].count);
    }
}

#[test]
fn disconnected_pins_all_actually_lack_a_net() {
    let db = Db::open(FIXTURE).unwrap();
    for entry in report::disconnected_pins(&db) {
        // each reported instance pin must genuinely carry no net
        if let Some((inst, pin)) = entry.split_once('/') {
            assert!(db.net_of(inst, pin).is_empty(), "{entry} was reported but has a net");
        }
    }
}

#[test]
fn verilog_header_has_module_and_ports() {
    let db = Db::open(FIXTURE).unwrap();
    let v = report::verilog_header(&db);
    assert!(v.starts_with("module counter ("), "header: {v}");
    assert!(v.trim_end().ends_with("endmodule"));
    for p in db.bterm_names() {
        assert!(v.contains(&p), "port {p} missing from header");
    }
}

#[test]
fn wire_length_runs() {
    // may be 0 on an unrouted fixture; must not panic and must be readable
    let _ = Db::open(FIXTURE).unwrap().total_wire_length();
}
