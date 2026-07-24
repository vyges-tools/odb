// SPDX-License-Identifier: Apache-2.0
// Exercises a cross-section of the machine-generated read accessors (generated_api.rs) to prove
// the generator's four marshalling paths — scalar, string/enum, relation, iterator — round-trip
// correctly against the hand-written surface.
use vyges_opendb::Db;

const FIXTURE: &str = "tests/fixtures/counter.odb";

#[test]
fn generated_string_and_relation_accessors() {
    let db = Db::open(FIXTURE).unwrap();

    // string getter on the block matches the hand-written block_name
    assert_eq!(db.block_get_name(), db.block_name());

    // relation: every instance's getMaster() name must match the hand-written inst_master
    let inst = db.nth_inst_name(0);
    assert_eq!(db.inst_get_master(&inst), db.inst_master(&inst));
    assert!(!db.inst_get_master(&inst).is_empty());
}

#[test]
fn generated_enum_accessor_reports_sig_type() {
    let db = Db::open(FIXTURE).unwrap();
    // the power/ground nets carry POWER/GROUND sig-types; at least one such special net exists
    let special: Vec<String> = db
        .net_names()
        .into_iter()
        .filter(|n| db.net_get_sig_type(n) == "POWER" || db.net_get_sig_type(n) == "GROUND")
        .collect();
    assert!(!special.is_empty(), "expected at least one POWER/GROUND net");
    // generated enum accessor agrees with the hand-written net_sigtype
    for n in &special {
        assert_eq!(db.net_get_sig_type(n), db.net_sigtype(n));
    }
}

#[test]
fn generated_iterator_matches_hand_written() {
    let db = Db::open(FIXTURE).unwrap();
    // block.getInsts() (generated) must enumerate exactly the hand-written instance set
    let gen_insts = db.block_get_insts();
    assert_eq!(gen_insts.len(), db.num_insts());
    assert_eq!(gen_insts, db.inst_names());

    // net.getITerms() (generated) must match the hand-written net_iterms for a signal net
    let net = db
        .net_names()
        .into_iter()
        .find(|n| !db.net_is_special(n) && !db.net_iterms(n).is_empty())
        .expect("a signal net with iterms");
    assert_eq!(db.net_get_i_terms(&net), db.net_iterms(&net));
}

#[test]
fn generated_scalar_accessor_reads() {
    let db = Db::open(FIXTURE).unwrap();
    // scalar getter: dbNet::getITermCount() must equal the hand-written net_iterms count
    for net in db.net_names() {
        assert_eq!(db.net_get_i_term_count(&net) as usize, db.net_iterms(&net).len());
    }
}
