from typing import NewType, TypeVar, Generic, Any
from mpyc import finfields

T = TypeVar('T', bound=Any)
N = TypeVar('N')

bn256_scalar_field_modulus = 21888242871839275222246405745257275088548364400416034343698204186575808495617
bls12_381_scalar_field_modulus = 52435875175126190479447740508185965837690552500527637822603658699938581184513
curve25519_scalar_field_modulus = 7237005577332262213973186563042994240857116359379907606001950938285454250989

field = None

def _set_modulus(value):
    global field
    if value == "bn256" or value == bn256_scalar_field_modulus:
        field = finfields.GF(bn256_scalar_field_modulus)
        return field
    elif value == "bls12_381" or value == bls12_381_scalar_field_modulus or value is None:
        field = finfields.GF(bls12_381_scalar_field_modulus)
        return field
    elif value == "curve25519" or value == curve25519_scalar_field_modulus:
        field = finfields.GF(curve25519_scalar_field_modulus)
        return field
    else:
        raise ValueError("The only supported scalar fields are those of the following curves: bn256, bls12_381, curve25519.")

class Public(Generic[T]):
    pass

class Private(Generic[T]):
    pass

class Array(Generic[T, N]):
    def __getitem__(self, key: int) -> T:
        return self[key]
    