
from zkpyc.types import Array, field # zk_ignore
from typing import Union, Any, List #zk_ignore
from math import floor, log2 #zk_ignore

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
    num = field.modulus + int(i) if int(i) < 0 else int(i) # type: ignore
    bits = [bool(int(digit)) for digit in bin(num)[2:]]
    length = len(bits)
    if length < N:
        return [False for _ in range(N - length)] + bits # type: ignore
    elif length == N:
        return bits # type: ignore
    else:
        return bits[-N:] # type: ignore


def pack(i) -> field:
    field_size = get_field_size()
    if len(i) > field_size:
        raise ValueError("Input length must be less than field modulus size")

    padded_i = [False] * (field_size - len(i)) + i
    num = int("".join(map(str, map(int, padded_i))), 2)
    original_value = num - field.modulus if num >= field.modulus else num # type: ignore

    return field(original_value) # type: ignore


def get_field_size() -> int:
    return floor(log2(field.modulus)) + 1 # type: ignore


sum_ = sum # zk_ignore
def sum(x: Array[Union[int, field], Any]) -> Union[int, field]:
    return sum_(x) # type: ignore
