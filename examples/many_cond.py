from zk_types import Private # zk_ignore

def main(x: Private[int]) -> int:
    return x*x if (5 < x < 10) and (x % 2 == 0) else 2