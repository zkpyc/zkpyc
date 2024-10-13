from zkpyc.types import Array, field # zk_ignore
from .lookup2bit import lookup

# Three-bit window lookup (2bits + signature bit) in 2bit table
# using two constraints. Maps the bits `b` to a list of constants `c`
def sel3s(b: Array[bool, 3], c: Array[field, 4]) -> field:
    alpha: field = lookup([b[0], b[1]], c)
    out: field = alpha - field(2) * (field(b[2]) if alpha != field(0) else field(0))
    return out