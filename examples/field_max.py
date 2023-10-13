from zk_types.types import Private, field # zk_ignore

def main(x: Private[field], y: Private[field]) -> field:
    return x if x > y else y
