from zk_types.types import Private, field # zk_ignore

def main(x: Private[field], y: Private[field]) -> field:
    assert(x != y)
    return x * y
