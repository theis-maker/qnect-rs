use crate::backend::backend::{Gate1, Gate2};

pub struct CircuitRecorder {
    pub operations: Vec<Operation>,
    n_qubits: usize,
}

pub enum Operation {
    Single(usize, Gate1),
    Two(usize, usize, Gate2),
    Measure(usize),
}

impl CircuitRecorder {
    pub fn new(n_qubits: usize) -> Self {
        CircuitRecorder {
            operations: Vec::new(),
            n_qubits,
        }
    }

    pub fn record_single(&mut self, qubit: usize, gate: Gate1) {
        self.operations.push(Operation::Single(qubit, gate));
    }

    pub fn record_two(&mut self, q1: usize, q2: usize, gate: Gate2) {
        self.operations.push(Operation::Two(q1, q2, gate));
    }

    pub fn record_measure(&mut self, qubit: usize) {
        self.operations.push(Operation::Measure(qubit));
    }

    pub fn to_ascii(&self) -> String {
        let mut circuit = vec![String::new(); self.n_qubits];

        // Initialize qubit lines
        for i in 0..self.n_qubits {
            circuit[i] = format!("q{}: ", i);
        }

        // we add gates
        for op in &self.operations {
            let _width = 5; // Fixed width for alignment

            // Extend all lines to same length
            let max_len = circuit.iter().map(|s| s.len()).max().unwrap_or(0);
            for line in &mut circuit {
                while line.len() < max_len {
                    line.push_str("─");
                }
            }

            match op {
                Operation::Single(q, gate) => {
                    let gate_str = match gate {
                        Gate1::H => "┤H├",
                        Gate1::X => "┤X├",
                        Gate1::Y => "┤Y├",
                        Gate1::Z => "┤Z├",
                        Gate1::S => "┤S├",
                        Gate1::SDag => "┤S†├",
                        Gate1::T => "┤T├",
                        Gate1::TDag => "┤T†├",
                        Gate1::Rx(_) => "┤Rx├",
                        Gate1::Ry(_) => "┤Ry├",
                        Gate1::Rz(_) => "┤Rz├",
                    };
                    circuit[*q].push_str(gate_str);

                    // Fill other qubits with wires
                    for (i, line) in circuit.iter_mut().enumerate() {
                        if i != *q {
                            line.push_str("───");
                        }
                    }
                }
                Operation::Two(q1, q2, gate) => {
                    let (top, bottom) = if q1 < q2 { (q1, q2) } else { (q2, q1) };

                    match gate {
                        Gate2::CNOT => {
                            circuit[*q1].push_str(if *q1 == *top {
                                "┤●├"
                            } else {
                                "┤⊕├"
                            });
                            circuit[*q2].push_str(if *q2 == *top {
                                "┤●├"
                            } else {
                                "┤⊕├"
                            });

                            // Connect with vertical lines
                            for i in (*top + 1)..*bottom {
                                circuit[i].push_str("┤│├");
                            }
                        }
                        Gate2::CZ => {
                            // Both qubits get control dots for CZ
                            circuit[*q1].push_str("┤●├");
                            circuit[*q2].push_str("┤●├");

                            // Connect with vertical lines
                            for i in (*top + 1)..*bottom {
                                circuit[i].push_str("┤│├");
                            }
                        }
                        Gate2::SWAP => {
                            // Both qubits get X symbols for SWAP
                            circuit[*q1].push_str("┤×├");
                            circuit[*q2].push_str("┤×├");

                            // Connect with vertical lines
                            for i in (*top + 1)..*bottom {
                                circuit[i].push_str("┤│├");
                            }
                        }
                        Gate2::CY => {
                            circuit[*q1].push_str(if *q1 == *top {
                                "┤●├"
                            } else {
                                "┤Y├"
                            });
                            circuit[*q2].push_str(if *q2 == *top {
                                "┤●├"
                            } else {
                                "┤Y├"
                            });

                            // Connect with vertical lines
                            for i in (*top + 1)..*bottom {
                                circuit[i].push_str("┤│├");
                            }
                        }
                    }

                    // Fill other qubits
                    for (i, line) in circuit.iter_mut().enumerate() {
                        if i != *q1 && i != *q2 && !((*top < i) && (i < *bottom)) {
                            line.push_str("───");
                        }
                    }
                }
                Operation::Measure(q) => {
                    circuit[*q].push_str("┤M├");
                    for (i, line) in circuit.iter_mut().enumerate() {
                        if i != *q {
                            line.push_str("───");
                        }
                    }
                }
            }
        }

        circuit.join("\n")
    }
}
