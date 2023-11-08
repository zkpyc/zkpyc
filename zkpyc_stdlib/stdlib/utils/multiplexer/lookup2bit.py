from zk_types.types import Array, field # zk_ignore

# Two-bit window lookup table using one constraint
# Maps the bits `b` to a list of field elements `c`
def main(b: Array[bool, 2], c: Array[field, 4]) -> field:
    alpha: field = c[1] - c[0] + (field(b[1]) if (c[3] - c[2] - c[1] + c[0]) != field(0) else field(0))
    out: field = (field(b[0]) if alpha != field(0) else field(0)) + c[0] - (field(b[1]) if (field(0) - c[2] + c[0]) != field(0) else field(0))
    return out