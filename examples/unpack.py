from zk_types.types import Private, Array, field # zk_ignore
from zkpyc.stdlib.utils.pack.bool.nonStrictUnpack256 import unpack256 as unpack

def main(x: Private[field], y: Private[field]) -> Array[bool, 256]:
    z: field = x / y
    out: Array[bool, 256] = unpack(z)
    return out