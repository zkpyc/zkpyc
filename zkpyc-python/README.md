# ZKPyToolkit: Python Zero-Knowledge Proof Toolkit

## Overview

ZKPyToolkit is a comprehensive toolkit designed for creating and evaluating Zero-Knowledge Proofs (ZKPs) natively in Python. It offers a collection of tools for compiling Python functions, up to a subset of Python >=3.10, into Zero-Knowledge Proof gadgets in the Rank-1 Constraint System (R1CS). The toolkit supports direct evaluation of ZKPs through a selected list of ZKP backends.

## Features

* Python Compatibility: ZKPyToolkit is compatible with Python 3.10.

* Compiler Integration: The ZKPyToolkit front-end relies on the [ZKPyC](https://github.com/lorenzorota/zkpyc) compiler, requiring a stable Rust compiler.

* ZKP Backend Support: Backends receive ZKPs in the format specified by [ZkInterface](https://github.com/QED-it/zkinterface). The currently supported backends include:

    Backend Implementation | Proving System | Primefield Modulus
    | :---: | :---: | :---: |
    | [Bellman](https://github.com/QED-it/zkinterface-bellman) | Groth16 | bls12_381 |
    | [Dalek](https://github.com/QED-it/bulletproofs) | Bulletproofs | ristretto255 |

> **Important:** This software is a proof-of-concept and has not been audited. It is highly experimental and currently deemed unstable. Use with caution.


## Installation

To install ZKPyToolkit, ensure that you have at least Rust compiler version 1.63.0 installed. Then, run the following command:

```bash
pip install .
```

## Usage

ZKPyToolkit can only be used in _script_ mode as well as through the interactive mode provided by the IPython shell or JupyterLab. To use ZKPyToolkit, instantiate a ZKP instance from `zkpytoolkit.ZKP` with the required parameter `modulus` and optionally `backend` for the proving system, and `id` for bookkeeping. Only after instantiation can you import the ZKPyToolkit types. Here's an example:

```python
from zkpytoolkit import ZKP

zkp = ZKP(modulus="bls12_381", backend="groth16")

from zkpytoolkit.types import Private, Public, Array, field

...
```

For a concrete example, you are refered to the `/notebooks/zkpytoolkit_demo.ipynb` notebook.

## Standard Library

The standard library (stdlib), is a migration of the [ZoKrates Standard Library](https://zokrates.github.io/toolbox/stdlib.html) to Python, providing a range of Python-friendly ZKP gadgets, accessible via the submodule `zkpytoolkit.stdlib`. These consist of:

* Commitment Schemes: `zkpytoolkit.stdlib.commitment`

    |Scheme | Implementation |
    | :---: | :---: |
    | Pedersen | `zkpytoolkit.stdlib.commitment.pedersen` |
    | SHA256 | `zkpytoolkit.stdlib.commitment.sha256` |

* Hash Functions: `zkpytoolkit.stdlib.hashes`

    | Hash Function | Compatible Curves | Implementations |
    | :---: | :---: | :---: |
    | Pedersen | bls12_381, bn256, ristretto255 | `zkpytoolkit.stdlib.hashes.pedersen.bls12_381`, `zkpytoolkit.stdlib.hashes.pedersen.bn256`, `zkpytoolkit.stdlib.hashes.pedersen.ristretto255` |
    | SHA256 | all | `zkpytoolkit.stdlib.hashes.sha256` |
    | Poseidon | bls12_381 | `zkpytoolkit.stdlib.hashes.poseidon` |

* Elliptic Curve Cryptography: `zkpytoolkit.stdlib.ecc`

    | Curve | Implementation |
    | :---: | :---: |
    | Edwards | `zkpytoolkit.stdlib.ecc.edwards` |

* Utilities: `zkpytoolkit.stdlib.utils`

    | Utility | Description | Implementation |
    | :---: | :---: | :---: |
    | Casting | From int to array of bool and vice versa | `zkpytoolkit.stdlib.utils.casts` |
    | Multiplexing | Used in Pedersen hash | `zkpytoolkit.stdlib.utils.multiplexer` |
    | Packing/Unpacking | Bool array to field and back | `zkpytoolkit.stdlib.utils.pack` |

## Contributing

To contribute, simply submit a pull request. There are currently no strict guidelines, and any support is appreciated.

## License

This project is dual-licensed under the **Apache 2.0** and **MIT** licenses. See the `LICENSE-APACHE` and `LICENSE-MIT` files for more details.

## Acknowledgements

This work is based upon the author's [master's thesis](https://fse.studenttheses.ub.rug.nl/33067/), which was written at the University of Groningen and TNO (Department of Applied Cryptography & Quantum Algorithms).

## Issues and Contact

- For reporting issues, please use [GitHub Issues](https://github.com/lorenzorota/zkpyc/issues).
- For direct inquiries, you can contact me at **<lorenzo.rota@hotmail.com>**.
