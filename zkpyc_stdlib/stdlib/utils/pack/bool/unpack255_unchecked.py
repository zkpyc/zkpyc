from zkpyc.types import Array, field # zk_ignore
from zkpyc.stdlib.EMBED import unpack

# Unpack a field element as 255 big endian bits without checking for overflows
# This does *not* guarantee a single output: for example, 0 can be decomposed as 0 or as P and this function does not enforce either
def unpack_unchecked(i: field) -> Array[bool, 255]:
    res: Array[bool, 255] = unpack(i, 255)
    return res