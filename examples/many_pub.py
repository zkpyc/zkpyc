from zk_types.types import Public # zk_ignore

def main(a: Public[int], b: Public[int], c: Public[int] , d: Public[int]) -> int:
    return a ^ b ^ c ^ d
