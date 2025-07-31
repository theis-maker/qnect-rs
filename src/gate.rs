#[derive(Debug, Clone, Copy)]
pub enum Gate {
    H,
    X,
    CX(usize, usize),
    Measure,
}
