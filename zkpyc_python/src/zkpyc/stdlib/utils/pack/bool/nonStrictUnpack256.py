from zkpyc.types import Array, field # zk_ignore
from .unpack255_unchecked import unpack_unchecked


# Unpack a field element as 256 big-endian bits
# Note: uniqueness of the output is not guaranteed
# For example, `0` can map to `[0, 0, ..., 0]` or to `bits(p)`
def unpack256(i: field) -> Array[bool, 256]:
    # b: Array[bool, 254] = unpack_unchecked(i)
    # return [False, False, *b]
    
    b: Array[bool, 255] = unpack_unchecked(i)
    return [False, *b]