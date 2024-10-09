from zk_types.types import Private # zk_ignore
from zkpyc.stdlib.hashes.sha256.shaRound import temp1

def main(e: Private[int], f: Private[int], g: Private[int], h: Private[int], k: Private[int], w: Private[int]) -> int:
    return temp1(e, f, g, h, k, w)
