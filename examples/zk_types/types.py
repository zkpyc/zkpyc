from typing import NewType, TypeVar, Generic
import mpyc
from mpyc.runtime import mpc

T = TypeVar('T')
N = TypeVar('N')

# field = NewType('field', int)

bls12_381_scalar_modulus = 21888242871839275222246405745257275088548364400416034343698204186575808495617

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