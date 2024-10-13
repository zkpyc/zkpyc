from zkpyc.types import Array # zk_ignore
from zkpyc.stdlib.EMBED import int_from_bits

def from_bits(a: Array[bool, 32]) -> int:
    return int_from_bits(a)