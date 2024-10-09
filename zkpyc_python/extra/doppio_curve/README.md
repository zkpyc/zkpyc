# Doppio curve

**derive.py** is a script for finding parameters for an Edwards curve embedded in Ristretto255. This program is a fork of the sagemath script available in [dalek-cryptography/doppio](https://github.com/dalek-cryptography/doppio).

## Purpose

The primary purpose of **derive.py** is to obtain the curve order for the Doppio group, which has the coefficients $a = 1$ and $d = -63071$. The resulting curve is of the order $1809251394333065553493296640760748560198346542606730328752412232071674536321$ and has cofactor 4. Since it is a modification of the script used to select the Jubjub parameters, it could be modified to find other embedded curves with special properties.

## Usage

To use **derive.py**, simply run the script:

```bash
sage -python derive.py
```
