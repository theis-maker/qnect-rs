# Changelog

## [0.2.0] - 2025-08-20

### Added
- Had a lot of code laying ready - pushing hard out
- Quantum Network Runtime - distributed quantum computing!
- Multi-hop entanglement routing with automatic swapping
- NetQASM code generation for real quantum hardware
- Mock QNPU backend for hardware API testing
- Blind quantum computation (UBQC protocol)
- Network topology visualization
- Production metrics and resource tracking

### Changed
- Made all operations async for distributed computing
- Enhanced examples with quantum networking demos

### Fixed
- Test suite now uses multi-threaded runtime

## [0.1.3] - 2025.08.19

### Added
- **Stabilizer Backend** - Simulate 5000+ qubit Clifford circuits with O(n²) memory
- **OpenQASM Support** - Import/export quantum circuits (QASM 2.0 stable, 3.0 experimental)
- **Noise Models** - Realistic quantum error simulation with depolarizing and measurement errors
- **Grover's Algorithm** - Complete implementation achieving 95% success rate
- **Three-qubit Gates** - Toffoli (CCX) gate support
- **Adjoint Gates** - S† and T† (s_dag, t_dag) gates
- **QASM Parser** - Full-featured parser supporting symbolic angles (pi/4, etc.)
- Comprehensive integration tests covering all features
- Examples 13-18 demonstrating new capabilities

### Changed
- Backend trait now supports swappable implementations (state vector, stabilizer)
- Builder pattern extended with `from_qasm()` and `with_noise()` methods
- Performance improvements in gate applications
- Enhanced error messages for backend-specific limitations
- Documentation updated to reflect 5000+ qubit capability

### Fixed
- Measurement statistics now properly random for superposition states
- Circuit recording handles all gate types correctly
- Error handling for invalid operations (same qubit CNOT, etc.)
- Noise model determinism with seeded RNG

### Performance
- Stabilizer backend: ~135,000 gates/second
- 5000 qubit GHZ state creation in ~2 seconds using only 12MB RAM
- State vector backend optimizations for up to 30 qubits

## [0.1.2] - 2025.08.18

### Added
- Comprehensive quantum verification example (`00_quantum_verification`)
- Circuit visualization with ASCII diagrams (`with_recording()`)
- CHSH inequality test with proper measurement angles
- Visual circuit output for all verification tests
- Better error messages for qubit out of range
- Example list command (`cargo run --example list`)

### Changed
- Improved README with correctness verification front and center
- Better performance documentation with realistic expectations
- Fixed CHSH measurement angles for optimal Bell inequality violation
- Enhanced circuit visualizer to show CZ, SWAP, and CY gates properly

### Fixed
- CHSH test now correctly violates Bell inequality
- Circuit visualization for two-qubit gates
