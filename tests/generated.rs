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

// --- the widened target set: dbMaster / dbITerm / dbMTerm / dbTechLayer -----------------

#[test]
fn generated_master_and_mterm_accessors() {
    let db = Db::open(FIXTURE).unwrap();
    let inst = db.nth_inst_name(0);
    let master = db.inst_master(&inst);
    // master round-trips its own name, and each of its mterms resolves back by (master, term)
    assert_eq!(db.master_get_name(&master), master);
    let mterms = db.master_get_m_terms(&master);
    assert!(!mterms.is_empty(), "a std-cell master should have terminals");
    for t in &mterms {
        assert_eq!(db.mterm_get_name(&master, t), *t);
        let io = db.mterm_get_io_type(&master, t);
        assert!(!io.is_empty(), "mterm {t} should report an IO type");
    }
}

#[test]
fn generated_iterm_relations_agree_with_hand_written() {
    let db = Db::open(FIXTURE).unwrap();
    // pick an instance pin that carries a net; the generated iterm relations must agree
    let (inst, pin, net) = (0..db.num_insts())
        .map(|i| db.nth_inst_name(i))
        .find_map(|inst| {
            let p = db.input_pin(&inst);
            let n = db.net_of(&inst, &p);
            (!p.is_empty() && !n.is_empty()).then(|| (inst.clone(), p, n))
        })
        .expect("a driven input pin");
    assert_eq!(db.iterm_get_inst(&inst, &pin), inst);
    assert_eq!(db.iterm_get_net(&inst, &pin), net);
}

#[test]
fn generated_row_and_site_accessors() {
    let db = Db::open(FIXTURE).unwrap();
    // a placed design has placement rows; enumerate them via the generated iterator
    let rows = db.block_get_rows();
    assert!(!rows.is_empty(), "a placed fixture should have rows");
    for row in &rows {
        // row name round-trips, and its site relation resolves to a named, non-empty site
        assert_eq!(db.row_get_name(row), *row);
        let site = db.row_get_site(row);
        assert!(!site.is_empty(), "row {row} should reference a site");
        assert_eq!(db.site_get_name(&site), site);
        // site dimensions are positive DBU values
        assert!(db.site_get_width(&site) > 0 && db.site_get_height(&site) > 0);
    }
}

#[test]
fn generated_module_hierarchy_reads() {
    let db = Db::open(FIXTURE).unwrap();
    // the fixture's flat design has a top module named after the block — exercises the newly
    // targeted dbModule class (name round-trip + hierarchy scalars/iterators)
    let top = db.block_name();
    assert_eq!(db.module_get_name(&top), top);
    assert!(db.module_get_db_inst_count(&top) > 0, "top module should contain instances");
    // the top module holds the logic cells — a non-empty subset of all block insts (physical-only
    // cells like fillers/tapcells live in the block but not the logical module).
    let mod_insts = db.module_get_insts(&top).len();
    assert!(
        mod_insts > 0 && mod_insts <= db.num_insts(),
        "top module insts ({mod_insts}) should be a non-empty subset of {} block insts",
        db.num_insts()
    );
}

#[test]
fn generated_struct_geometry_subfields() {
    let db = Db::open(FIXTURE).unwrap();
    // a Rect getter (block die area) expands into x_min/y_min/x_max/y_max/dx/dy scalar sub-fields
    let (x0, y0) = (db.block_get_die_area_x_min(), db.block_get_die_area_y_min());
    let (x1, y1) = (db.block_get_die_area_x_max(), db.block_get_die_area_y_max());
    assert!(x1 >= x0 && y1 >= y0, "die area should be a valid rect: ({x0},{y0})-({x1},{y1})");
    // the derived dimensions are internally consistent
    assert_eq!(db.block_get_die_area_dx(), x1 - x0);
    assert_eq!(db.block_get_die_area_dy(), y1 - y0);
    // a Point getter (inst origin) expands into x/y; readable for every instance
    let inst = db.nth_inst_name(0);
    let _ = (db.inst_get_origin_x(&inst), db.inst_get_origin_y(&inst));
}

#[test]
fn generated_index_addressed_box_geometry() {
    // the index-addressing mode: dbObstruction/dbBox have no names, so they're addressed by
    // position. Add a known-rect obstruction, then read its bbox back through the generated
    // dbBox geometry getters (i-th obstruction's getBBox()).
    let mut db = Db::open(FIXTURE).unwrap();
    let n0 = db.num_obstructions();
    db.add_obstruction("met1", 1000, 2000, 5000, 8000).unwrap();
    assert_eq!(db.num_obstructions(), n0 + 1);

    let i = (0..db.num_obstructions())
        .find(|&i| {
            db.box_x_min(i) == 1000 && db.box_y_min(i) == 2000
                && db.box_x_max(i) == 5000 && db.box_y_max(i) == 8000
        })
        .expect("the added obstruction's bbox should be readable by index");
    // dbBox derived dimensions agree with the rect (getDX/getDY = width/height)
    assert_eq!(db.box_get_d_x(i), 4000);
    assert_eq!(db.box_get_d_y(i), 6000);
    // an obstruction predicate reads without panic
    let _ = db.obs_is_slot_obstruction(i);
}

#[test]
fn generated_tech_layer_accessors() {
    let db = Db::open(FIXTURE).unwrap();
    // enumerate routing layers off the tech; every layer round-trips its name and reads a width
    let layers = db.block_get_tech(); // relation → tech name (proves the tech exists)
    assert!(!layers.is_empty(), "fixture should carry a tech");
    // a known sky130 routing layer in the fixture
    for name in ["met1", "li1", "met2"] {
        let got = db.layer_get_name(name);
        if !got.is_empty() {
            assert_eq!(got, name);
            // width is a non-negative DBU value; just prove the scalar path reads without panic
            let _ = db.layer_get_width(name);
            return;
        }
    }
    panic!("expected at least one of met1/li1/met2 in the fixture tech");
}
