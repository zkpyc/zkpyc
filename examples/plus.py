from zk_types.types import Private, Public, field # zk_ignore

def main(a: Private[field], b: Private[field]) -> field:
    return a + b