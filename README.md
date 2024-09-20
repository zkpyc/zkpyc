<img src="zkpyc-logo.svg" width="100%" height="180">

# ZKPyC - The Zero-Knowledge Proof Compiler for Python

ZKPyC is a compiler for a subset of Python 3.10 to a Rank-1 Constraint System (R1CS) description, used by modern zero-knowledge proof systems such as zk-SNARKs. The ZKPyC compiler leverages the [CirC](https://github.com/circify/circ) circuit compiler infrastructure to produce optimized and secure R1CS constraints.

## Features

- Compile Python 3.10 code (subset defined in `zkpyc.asdl`) to R1CS.
- Export R1CS in two formats:
  1. **CirC-IR serialization**: Compatible with CirC backends like Groth16, Mirage, and Spartan.
  2. **zkInterface format**: Compatible with zkInterface-compatible backends.
- Generate valid witnesses for the R1CS constraints.

> **Note:** This software is a proof-of-concept and has not been audited. Use with caution

## Installation

To install ZKPyC, you need a stable Rust compiler. Install directly from GitHub using the following command:

```bash
cargo install --git https://github.com/lorenzorota/zkpyc.git zkpyc
```

## Usage

### Compiling Python Code to R1CS

To compile a Python file into an instance of R1CS:

```bash
zkpyc <file_name> r1cs --action setup --proof-impl <groth16 | zkinterface>
```

### Generating Witness

To generate the witness for the corresponding R1CS instance:

```bash
zk --inputs <prover_inputs_file_name> --action prove --proof-impl <groth16 | zkinterface>
```

### Verifying Witness

To verify the witness or generate the verifier's zkInterface file:

```bash
zk --inputs <verifier_inputs_file_name> --action verify --proof-impl <groth16 | zkinterface>
```

## Example Workflow

### Groth16 Back-End

```bash
# Compile the program into an R1CS instance
zkpyc examples/mm.py r1cs --action setup --proof-impl groth16

# Generate the zero-knowledge proof
zk --inputs examples/mm.py.pin --action prove --proof-impl groth16

# Verify the witness in zero-knowledge
zk --inputs examples/mm.py.vin --action verify --proof-impl groth16
```

### zkInterface Back-End (Ristretto255 Scalar Field)

```bash
# Compile the program into an R1CS instance
zkpyc --field-custom-modulus 7237005577332262213973186563042994240857116359379907606001950938285454250989 examples/zkinterface.py --action setup --proof-impl zkinterface

# Generate the zero-knowledge proof
zk --field-custom-modulus 7237005577332262213973186563042994240857116359379907606001950938285454250989 examples/zkinterface.py.pin --action prove --proof-impl zkinterface

# Verify the witness in zero-knowledge
zk --field-custom-modulus 7237005577332262213973186563042994240857116359379907606001950938285454250989 examples/zkinterface.py.vin --action verify --proof-impl zkinterface
```

## Contributing

To contribute, simply submit a pull request. There are currently no strict guidelines, and any support is appreciated.

## License

This project is dual-licensed under the **Apache 2.0** and **MIT** licenses. See the `LICENSE-APACHE` and `LICENSE-MIT` files for more details.

> **Note**: ZKPyC is primarily a front-end for the [CirC](https://github.com/circify/circ) project and involves modifications of the CirC-ZoKrates implementation. The compiler relies on the [RustPython parser](https://github.com/RustPython/Parser) for translating Python code into an abstract syntax tree. Lastly, the standard library (stdlib) in this project is a migration of the [ZoKrates standard library](https://github.com/Zokrates/ZoKrates/tree/develop/zokrates_stdlib) to Python, with some small additions and omissions.

## Acknowledgements

This work is based upon the author's [master's thesis](https://fse.studenttheses.ub.rug.nl/33067/), which was written at the University of Groningen and TNO (Department of Applied Cryptography & Quantum Algorithms).

The logo used in this repository, `zkpyc-logo.svg`, is a derivative of the official Python logo. Usage of the derived logo has been approved by the Python Software Foundation (PSF) under the terms of their [trademark usage policy](https://www.python.org/psf/trademarks/).

## Issues and Contact

- For reporting issues, please use [GitHub Issues](https://github.com/lorenzorota/zkpyc/issues).
- For direct inquiries, you can contact me at **<lorenzo.rota@hotmail.com>**.
