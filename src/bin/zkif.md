
# Usage

To use zkif, first generate the three files: `header.zkif`, `constraints.zkif` and `witness.zkif`.

```bash
zsh scripts/prove_r1cs_py_test.zsh
```

Example (Groth16):

```bash
zkif cat header.zkif constraints_0.zkif | zkif_bellman setup
zkif cat header.zkif constraints_0.zkif witness.zkif | zkif_bellman prove
zkif cat header.zkif constraints_0.zkif | zkif_bellman verify
```

Example (Bulletproofs):

```bash
zkif cat header.zkif constraints_0.zkif witness.zkif | zkif_bulletproofs prove
zkif cat header.zkif constraints_0.zkif | zkif_bulletproofs verify
```
