from typing import NewType, TypeVar, Generic, Any
from mpyc import finfields

T = TypeVar('T', bound=Any)
N = TypeVar('N')

# field = NewType('field', int)

bn256_scalar_modulus = 21888242871839275222246405745257275088548364400416034343698204186575808495617
bls12_381_scalar_modulus = 52435875175126190479447740508185965837690552500527637822603658699938581184513
curve25519_scalar_field_modulus = 7237005577332262213973186563042994240857116359379907606001950938285454250989

# field = finfields.GF(bn256_scalar_modulus)
field = finfields.GF(bls12_381_scalar_modulus)
# field = finfields.GF(curve25519_scalar_field_modulus)

class Public(Generic[T]):
    pass

class Private(Generic[T]):
    pass

class Array(Generic[T, N]):
    def __getitem__(self, key: int) -> T:
        return self[key]
