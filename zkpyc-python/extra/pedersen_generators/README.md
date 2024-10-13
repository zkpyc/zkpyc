# Pedersen Generators

**ecc_params** is a Python tool designed for generating Windowed Pedersen Hash lookup tables of two generator base points over the embedded curves JubJub, Baby JubJub, and Doppio. These lookup tables can be used in gadgets for zero-knowledge proof (ZKP) backends utilizing the BLS12_381, BN256, and Ristretto255 curves, respectively. This tool is a fork of [Zokrates/pycrypto](https://github.com/Zokrates/pycrypto).

## Usage

To generate tables for specific curves, run one of the following options:

```bash
# To generate lookup tables for BLS12_381
python run.py bls12_381

# To generate lookup tables for BN256
python run.py bn256

# To generate lookup tables for Ristretto255
python run.py ristretto255
```


This will produce the source code defining two lookup tables from uncorrelated generator points, equivalent to the generator elements H and G from the standard definition of the Pedersen commitment scheme.

## Customizing Parameters

To generate tables for other curves, simply modify the parameters of the Edwards curve in `run.py`.
