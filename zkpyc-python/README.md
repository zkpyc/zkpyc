# The ZKPyC Python Package

> **Important:** This software is a proof-of-concept and has not been audited. It is highly experimental and subject to changes. Please use it with caution!

## Overview

The ZKPyC Python package provides developers with a powerful and intuitive toolkit for generating Zero-Knowledge Proofs (ZKPs) directly from Python code. It is designed for a strict subset of Python and enables the creation of ZKPs while abstracting away the complexity of producing low-level ZKP gadgets and generating proofs.

## Features

* **Python Integration**: Compatible with a strict subset of Python >= 3.10, as described in the ZKPyC documentation.

* **Compiler Integration**: Uses the [ZKPyC](https://github.com/lorenzorota/zkpyc) compiler, requiring a stable version of Rust.

* **ZKP Backend Support**: Outputs proofs in the [ZkInterface](https://github.com/QED-it/zkinterface) format. The currently supported proof systems are:

    Backend | Proof System | Primefield Modulus
    :---: | :---: | :---:
    [Bellman](https://github.com/QED-it/zkinterface-bellman) | Groth16 | bls12_381
    [Dalek](https://github.com/QED-it/bulletproofs) | Bulletproofs | ristretto255

* **Standard Library**: A standard library adapted from the [ZoKrates Standard Library](https://zokrates.github.io/toolbox/stdlib.html), which includes:

  * **Hash Functions**: Pedersen, SHA256, Poseidon.
  * **Elliptic Curve Operations**: For the Jubjub, Baby-Jubjub and Doppio curves in the Edwards form.
  * **Utilities**: Packing, unpacking, casting, and multiplexing.

## Installation

To install the ZKPyC package, ensure that you have at least Rust compiler version 1.63.0 installed. Then, run the following command:

```bash
pip install .
```

## Usage

ZKPyC can be used in script mode or interactively through IPython or JupyterLab. To get started, instantiate a ZKP object from zkpyc.ZKP by providing the required parameter modulus. Optional parameters include backend for selecting a proving system and id for bookkeeping. Importing ZKP types is only possible after instantiating a ZKP object.

Here's an example:

```python
from zkpyc import ZKP
zkp = ZKP(modulus="bls12_381", backend="groth16")
from zkpyc.types import Private, Public, Array, field

...
```

For concrete and interactive examples, visit the [demos](./demos) directory.

## Standard Library

The standard library, `zkpyc.stdlib`, is a Python adaptation of the ZoKrates Standard Library, providing useful cryptographic functions and utilities that are compatible with ZKPyC. It is bundled with the ZKPyC compiler and maintained as part of the [zkpyc-stdlib](https://github.com/zkpyc/zkpyc/tree/main/zkpyc-stdlib) crate. For additional details, visit the linked repository.

## Contributing

To contribute, simply submit a pull request. There are currently no strict guidelines, and any support is appreciated.

## License

This project is dual-licensed under the **Apache 2.0** and **MIT** licenses. See the `LICENSE-APACHE` and `LICENSE-MIT` files for more details.

## Acknowledgements

This project is part of the overarching [ZKPyC repository](https://github.com/zkpyc/zkpyc).

## Issues and Contact

* For reporting issues, please use [GitHub Issues](https://github.com/lorenzorota/zkpyc/issues).
* For direct inquiries, you can contact me at **<lorenzo.rota@hotmail.com>**.
