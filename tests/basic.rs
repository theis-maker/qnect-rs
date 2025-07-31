use qnect::circuit::Circuit;

#[test]
fn build_simple_circuit() {
    let mut circuit = Circuit::new(2);
    circuit.h(0);
    circuit.cx(0, 1);
    circuit.measure_all();

    circuit.print(); // Just to show it works
}
