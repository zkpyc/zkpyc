from zk_types.types import Array, field # zk_ignore
from EMBED import unpack;

# Unpack a field element as N big endian bits without checking for overflows
# This does *not* guarantee a single output: for example, 0 can be decomposed as 0 or as P and this function does not enforce either
def main(i: field) -> Array[bool, 254]:
    res: Array[bool, 254] = unpack(i, 254)
    return res