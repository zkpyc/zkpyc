from typing import NewType, TypeVar, Generic, Any
import mpyc
from mpyc.runtime import mpc

T = TypeVar('T', bound=Any)
N = TypeVar('N')

# field = NewType('field', int)

bn256_scalar_modulus = 21888242871839275222246405745257275088548364400416034343698204186575808495617
bls12_381_scalar_modulus = 52435875175126190479447740508185965837690552500527637822603658699938581184513

# field = mpyc.finfields.GF(bn256_scalar_modulus)
field = mpyc.finfields.GF(bls12_381_scalar_modulus)

class Public(Generic[T]):
    pass

class Private(Generic[T]):
    pass

class Array(Generic[T, N]):
    def __getitem__(self, key: int) -> T:
        return self[key]

# Public = NewType('Public', Array)

# Private = NewType('Private', Array)