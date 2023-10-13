from examples.zk_types.types import Private # zk_ignore

def main(x: Private[int]) -> int:
    return x + x + x
