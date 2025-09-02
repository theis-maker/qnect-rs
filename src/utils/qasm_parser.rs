use crate::error::{QnectError, Result};

#[derive(Debug)]
pub struct QasmProgram {
    pub version: String,
    pub num_qubits: usize,
    pub num_bits: usize,
    pub operations: Vec<QasmOperation>,
}

#[derive(Debug, Clone)]
pub enum QasmOperation {
    H(usize),
    X(usize),
    Y(usize),
    Z(usize),
    S(usize),
    SDag(usize),
    T(usize),
    TDag(usize),
    Rx(usize, f64),
    Ry(usize, f64),
    Rz(usize, f64),
    CX(usize, usize),
    CY(usize, usize),
    CZ(usize, usize),
    Swap(usize, usize),
    Measure(usize, usize), // (qubit, bit)
}

pub fn parse_qasm(qasm: &str) -> Result<QasmProgram> {
    let mut lines = qasm
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"));

    // Check version
    let version_line = lines.next().ok_or_else(|| {
        QnectError::invalid_operation("parse_qasm".to_string(), "Empty QASM file".to_string())
    })?;

    let version = if version_line.starts_with("OPENQASM 2.0") {
        "2.0".to_string()
    } else if version_line.starts_with("OPENQASM 3.0") {
        "3.0".to_string()
    } else {
        return Err(QnectError::invalid_operation(
            "parse_qasm".to_string(),
            "Unsupported QASM version".to_string(),
        ));
    };

    let mut num_qubits = 0;
    let mut num_bits = 0;
    let mut operations = Vec::new();

    for line in lines {
        // Skip include statements
        if line.starts_with("include") {
            continue;
        }

        // Parse quantum register
        if line.starts_with("qreg") || line.starts_with("qubit[") {
            num_qubits = parse_register_size(line)?;
            continue;
        }

        // Parse classical register
        if line.starts_with("creg") || line.starts_with("bit[") {
            num_bits = parse_register_size(line)?;
            continue;
        }

        // Parse gates and measurements
        if let Some(op) = parse_operation(line)? {
            operations.push(op);
        }
    }

    Ok(QasmProgram {
        version,
        num_qubits,
        num_bits,
        operations,
    })
}

fn parse_register_size(line: &str) -> Result<usize> {
    // Handle both "qreg q[3];" and "qubit[3] q;" formats
    let size_str = line
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .ok_or_else(|| {
            QnectError::invalid_operation(
                "parse_register_size".to_string(),
                "Invalid register declaration".to_string(),
            )
        })?;

    size_str.parse::<usize>().map_err(|_| {
        QnectError::invalid_operation(
            "parse_register_size".to_string(),
            "Invalid register size".to_string(),
        )
    })
}

fn parse_operation(line: &str) -> Result<Option<QasmOperation>> {
    let line = line.trim_end_matches(';').trim();

    // Skip empty lines
    if line.is_empty() {
        return Ok(None);
    }

    // Parse measurement: "measure q[0] -> c[0]" or "c[0] = measure q[0]"
    if line.contains("measure") {
        return parse_measurement(line).map(Some);
    }

    // Parse gates
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(None);
    }

    let gate_name = parts[0];

    match gate_name {
        "h" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for H gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::H(qubit)))
        }
        "x" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for X gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::X(qubit)))
        }
        "y" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for Y gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::Y(qubit)))
        }
        "z" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for Z gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::Z(qubit)))
        }
        "s" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for S gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::S(qubit)))
        }
        "sdg" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for S† gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::SDag(qubit)))
        }
        "t" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for T gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::T(qubit)))
        }
        "tdg" => {
            let qubit = parse_qubit_index(parts.get(1).ok_or_else(|| {
                QnectError::invalid_operation(
                    "parse_operation".to_string(),
                    "Missing qubit for T† gate".to_string(),
                )
            })?)?;
            Ok(Some(QasmOperation::TDag(qubit)))
        }
        "cx" => parse_two_qubit_gate(line, QasmOperation::CX),
        "cy" => parse_two_qubit_gate(line, QasmOperation::CY),
        "cz" => parse_two_qubit_gate(line, QasmOperation::CZ),
        "swap" => parse_two_qubit_gate(line, QasmOperation::Swap),
        _ if gate_name.starts_with("rx(") => parse_rotation_gate(line, "rx", QasmOperation::Rx),
        _ if gate_name.starts_with("ry(") => parse_rotation_gate(line, "ry", QasmOperation::Ry),
        _ if gate_name.starts_with("rz(") => parse_rotation_gate(line, "rz", QasmOperation::Rz),
        _ => Err(QnectError::invalid_operation(
            "parse_operation".to_string(),
            format!("Unknown gate: {}", gate_name),
        )),
    }
}

fn parse_qubit_index(qubit_str: &str) -> Result<usize> {
    // Parse "q[0]" or similar
    let index_str = qubit_str
        .split('[')
        .nth(1)
        .and_then(|s| s.split(']').next())
        .ok_or_else(|| {
            QnectError::invalid_operation(
                "parse_qubit_index".to_string(),
                "Invalid qubit format".to_string(),
            )
        })?;

    index_str.parse::<usize>().map_err(|_| {
        QnectError::invalid_operation(
            "parse_qubit_index".to_string(),
            "Invalid qubit index".to_string(),
        )
    })
}

fn parse_two_qubit_gate<F>(line: &str, constructor: F) -> Result<Option<QasmOperation>>
where
    F: Fn(usize, usize) -> QasmOperation,
{
    // Parse "cx q[0], q[1]" or "cx q[0],q[1]"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(QnectError::invalid_operation(
            "parse_two_qubit_gate".to_string(),
            "Invalid two-qubit gate".to_string(),
        ));
    }

    // Handle both "q[0], q[1]" and "q[0],q[1]"
    let qubits_str = parts[1..].join(" ");
    let qubits_str = qubits_str.replace(", ", ",");
    let qubit_parts: Vec<&str> = qubits_str.split(',').collect();

    if qubit_parts.len() != 2 {
        return Err(QnectError::invalid_operation(
            "parse_two_qubit_gate".to_string(),
            "Two-qubit gate needs exactly 2 qubits".to_string(),
        ));
    }

    let q1 = parse_qubit_index(qubit_parts[0].trim())?;
    let q2 = parse_qubit_index(qubit_parts[1].trim())?;

    Ok(Some(constructor(q1, q2)))
}

fn parse_rotation_gate<F>(
    line: &str,
    _gate_type: &str,
    constructor: F,
) -> Result<Option<QasmOperation>>
where
    F: Fn(usize, f64) -> QasmOperation,
{
    // Parse "rx(pi/2) q[0]" or "rx(1.5707963267948966) q[0]"
    let angle_start = line.find('(').ok_or_else(|| {
        QnectError::invalid_operation(
            "parse_rotation_gate".to_string(),
            "Missing angle in rotation gate".to_string(),
        )
    })?;
    let angle_end = line.find(')').ok_or_else(|| {
        QnectError::invalid_operation(
            "parse_rotation_gate".to_string(),
            "Missing closing parenthesis in rotation gate".to_string(),
        )
    })?;

    let angle_str = &line[angle_start + 1..angle_end];
    let angle = parse_angle(angle_str)?;

    let qubit_part = line[angle_end + 1..].trim();
    let qubit = parse_qubit_index(qubit_part)?;

    Ok(Some(constructor(qubit, angle)))
}

fn parse_angle(angle_str: &str) -> Result<f64> {
    use std::f64::consts::PI;

    // Handle symbolic pi expressions
    let angle_str = angle_str.trim();

    // Direct numeric value
    if let Ok(val) = angle_str.parse::<f64>() {
        return Ok(val);
    }

    // Handle pi expressions
    match angle_str {
        "0" => Ok(0.0),
        "pi" => Ok(PI),
        "pi/2" => Ok(PI / 2.0),
        "pi/4" => Ok(PI / 4.0),
        "pi/3" => Ok(PI / 3.0),
        "pi/6" => Ok(PI / 6.0),
        "pi/8" => Ok(PI / 8.0),
        "2*pi" => Ok(2.0 * PI),
        "3*pi/2" => Ok(3.0 * PI / 2.0),
        "3*pi/4" => Ok(3.0 * PI / 4.0),
        "5*pi/4" => Ok(5.0 * PI / 4.0),
        "7*pi/4" => Ok(7.0 * PI / 4.0),
        "2*pi/3" => Ok(2.0 * PI / 3.0),
        "5*pi/6" => Ok(5.0 * PI / 6.0),
        _ => Err(QnectError::invalid_operation(
            "parse_angle".to_string(),
            format!("Unknown angle expression: {}", angle_str),
        )),
    }
}

fn parse_measurement(line: &str) -> Result<QasmOperation> {
    // Handle both QASM 2.0 "measure q[0] -> c[0]" and QASM 3.0 "c[0] = measure q[0]"

    if line.contains("->") {
        // QASM 2.0 format
        let parts: Vec<&str> = line.split("->").collect();
        if parts.len() != 2 {
            return Err(QnectError::invalid_operation(
                "parse_measurement".to_string(),
                "Invalid measurement syntax".to_string(),
            ));
        }

        let qubit_part = parts[0].trim().replace("measure", "").trim().to_string();
        let bit_part = parts[1].trim();

        let qubit = parse_qubit_index(&qubit_part)?;
        let bit = parse_qubit_index(bit_part)?; // Reuse for bit index

        Ok(QasmOperation::Measure(qubit, bit))
    } else if line.contains("=") {
        // QASM 3.0 format
        let parts: Vec<&str> = line.split("=").collect();
        if parts.len() != 2 {
            return Err(QnectError::invalid_operation(
                "parse_measurement".to_string(),
                "Invalid measurement syntax".to_string(),
            ));
        }

        let bit_part = parts[0].trim();
        let qubit_part = parts[1].trim().replace("measure", "").trim().to_string();

        let bit = parse_qubit_index(bit_part)?;
        let qubit = parse_qubit_index(&qubit_part)?;

        Ok(QasmOperation::Measure(qubit, bit))
    } else {
        Err(QnectError::invalid_operation(
            "parse_measurement".to_string(),
            "Invalid measurement format".to_string(),
        ))
    }
}
