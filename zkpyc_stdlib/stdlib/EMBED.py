
from zk_types.types import Array, field # zk_ignore
from typing import Any, List #zk_ignore

# These functions are not run by ZKPyC as they are handled internally.
# They do however need to be defined for the python runtime.

def int_to_bits(n: int) -> Array[bool, Any]:
    bits = [bool(int(digit)) for digit in bin(n)[2:]]
    length = len(bits)
    if length < 32:
        return [False]*(32 - length) + bits # type: ignore
    elif length == 32:
        return bits # type: ignore
    else:
        return bits[-32:] # type: ignore


def int_from_bits(bits: Array[bool, Any]) -> int:
    result = 0
    for bit in bits: # type: ignore
        result = (result << 1) | bit
    return result


def unpack(i: field, N: int) -> Array[bool, Any]:
    num = field.modulus + int(i) if int(i) < 0 else int(i)
    bits = [bool(int(digit)) for digit in bin(num)[2:]]
    length = len(bits)
    if length < N:
        return [False for _ in range(N - length)] + bits # type: ignore
    elif length == N:
        return bits # type: ignore
    else:
        return bits[-N:] # type: ignore