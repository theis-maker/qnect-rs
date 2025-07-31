use crate::gate::Gate;
use crate::qubit::Qubit;

#[derive(Debug)]
pub struct Circuit {
    pub qubits: Vec<Qubit>,
    pub gates: Vec<Gate>,
}

impl Circuit {
    pub fn new(num_qubits: usize) -> Self {
        let qubits = (0..num_qubits).map(Qubit::new).collect();
        Circuit {
            qubits,
            gates: Vec::new(),
        }
    }

    pub fn h(&mut self, target: usize) {
        self.gates.push(Gate::H);
    }

    pub fn x(&mut self, target: usize) {
        self.gates.push(Gate::X);
    }

    pub fn cx(&mut self, control: usize, target: usize) {
        self.gates.push(Gate::CX(control, target));
    }

    pub fn measure_all(&mut self) {
        self.gates.push(Gate::Measure);
    }

    pub fn print(&self) {
        println!("Qubits: {:?}", self.qubits);
        println!("Gates: {:?}", self.gates);
    }
}
