use std::collections::{HashMap, HashSet};

/// Tracks entanglement relationships between qubits
#[derive(Debug, Clone)]
pub struct EntanglementTracker {
    /// Maps each qubit to its entanglement group
    /// Qubits in the same group are potentially entangled
    groups: HashMap<usize, usize>,
    /// Next available group ID
    next_group_id: usize,
}

impl EntanglementTracker {
    pub fn new(n_qubits: usize) -> Self {
        let mut groups = HashMap::new();
        // Initially, each qubit is in its own group
        for i in 0..n_qubits {
            groups.insert(i, i);
        }

        EntanglementTracker {
            groups,
            next_group_id: n_qubits,
        }
    }

    /// Mark two qubits as potentially entangled
    pub fn entangle(&mut self, qubit1: usize, qubit2: usize) {
        let group1 = self.groups[&qubit1];
        let group2 = self.groups[&qubit2];

        if group1 != group2 {
            // Merge groups: move all qubits from group2 to group1
            for (_q, g) in self.groups.iter_mut() {
                if *g == group2 {
                    *g = group1;
                }
            }
        }
    }

    /// Check if two qubits are potentially entangled
    pub fn are_entangled(&self, qubit1: usize, qubit2: usize) -> bool {
        self.groups.get(&qubit1) == self.groups.get(&qubit2)
    }

    /// Get all qubits entangled with a given qubit
    pub fn get_entangled_qubits(&self, qubit: usize) -> HashSet<usize> {
        let group = match self.groups.get(&qubit) {
            Some(g) => *g,
            None => return HashSet::new(),
        };

        self.groups
            .iter()
            .filter(|(_, g)| **g == group)
            .map(|(q, _)| *q)
            .collect()
    }

    /// Mark a qubit as measured (breaks entanglement)
    pub fn measure(&mut self, qubit: usize) {
        // After measurement, qubit is in its own group
        self.groups.insert(qubit, self.next_group_id);
        self.next_group_id += 1;
    }
}
