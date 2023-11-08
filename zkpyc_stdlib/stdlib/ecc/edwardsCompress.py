from zk_types.types import Array, field # zk_ignore
from utils.pack.bool.nonStrictUnpack256 import main as unpack256

# Compress JubJub Curve Point to 256bit array using big endianness bit order

def main(pt: Array[field, 2])  -> Array[bool, 256]:
    x: field = pt[0]
    y: field = pt[1]

    xBits: Array[bool, 256] = unpack256(x)
    yBits: Array[bool, 256] = unpack256(y)

    sign: bool = xBits[255]
    yBits[0] = sign

    return yBits