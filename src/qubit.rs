#[derive(Debug, Clone)]
pub struct Qubit {
    pub id: usize,
}

impl Qubit {
    pub fn new(id: usize) -> Self {
        Qubit { id }
    }
}
