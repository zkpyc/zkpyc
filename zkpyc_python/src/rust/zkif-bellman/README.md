# zkInterface Bellman adapter

More on zkInterface: https://github.com/QED-it/zkinterface

More on Bellman: https://github.com/zcash/librustzcash

## Usage

Bellman prover.

Validate that the witness satisfies the constraints:

    zkif_bellman validate

Print the circuit in a text-form:

    zkif_bellman print

Generate public parameters:

    zkif_bellman setup <workspace>

Generate a proof using the public parameters:

    zkif_bellman prove <workspace>

The circuit and witness are read from stdin in zkInterface format.
The filenames of keys and proofs are derived from the workspace argument; defaults to the current directory.

## Example:

Create a proving key:

    cat src/demo_import_from_zokrates/messages/*.zkif | cargo run --release setup

Create a proof:

    cat src/demo_import_from_zokrates/messages/*.zkif | cargo run --release prove

## Modifications

This is a fork of the [QED-it/zkinterface-bellman](https://github.com/QED-it/zkinterface-bellman) repository (commit ec9c232), which implements a ZK-interface adapter ontop of the Bellman's groth16 implementation. No further modifications were made ot the code base

Usage:

```bash
zkif_bulletproofs prove  [generators_count] [proof_path]
zkif_bulletproofs verify [generators_count] [proof_path]
```

Example:

```bash
zkif cat header.zkif constraints_0.zkif | zkif_bellman setup
zkif cat header.zkif constraints_0.zkif witness.zkif | zkif_bellman prove
zkif cat header.zkif constraints_0.zkif | zkif_bellman verify
```

The proper way to build this project is by entering:

```bash
cargo +nightly install --path .
```
