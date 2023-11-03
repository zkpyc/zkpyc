from zk_types.types import Private, Public, field # zk_ignore

def main(x: Public[field], y: Private[field]) -> field:
    xx: field = x * x
    yy: field = y * y
    return xx + yy - field(1)