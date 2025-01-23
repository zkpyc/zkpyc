from zk_types.types import Private # zk_ignore

def main(x: Private[int], y: Private[int]) -> int:
    return x // y
