use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

pub type BellId = u64;
pub type GhzId = u64;

#[derive(Debug, Clone, Copy, Default)]
pub struct PauliBits {
    pub ax: u8, // X component
    pub az: u8, // Z component
}

#[derive(Debug)]
pub struct SharedBell {
    pub id: BellId,
    pub left: (String, usize),
    pub right: (String, usize),
    pub left_pf: PauliBits,
    pub right_pf: PauliBits,
    pub alive: bool,
}

#[derive(Debug)]
pub struct SharedGhz {
    pub id: GhzId,
    pub parties: Vec<(String, usize)>,
    pub phase_bit: u8,
    pub measured: HashMap<(String, usize), u8>,
    pub pending_x_basis: HashSet<(String, usize)>,
}

#[derive(Default)]
pub struct NonlocalStore {
    pub next_bell_id: BellId,
    pub next_ghz_id: GhzId,
    pub bell_by_qubit: HashMap<(String, usize), BellId>,
    pub ghz_by_qubit: HashMap<(String, usize), GhzId>,
    pub bells: HashMap<BellId, Arc<Mutex<SharedBell>>>,
    pub ghzs: HashMap<GhzId, Arc<Mutex<SharedGhz>>>,
}
