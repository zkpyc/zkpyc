from zk_types.types import Private, field # zk_ignore

def main(A: Private[field], B: Private[field]) -> bool:
    assert(A+B == field(123))
    return True
