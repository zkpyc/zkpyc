from zkpyc.types import Array # zk_ignore
from zkpyc.stdlib.EMBED import int_to_bits

def to_bits(a: int) -> Array[bool, 32]:
    return int_to_bits(a)