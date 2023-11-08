from zk_types.types import Array # zk_ignore
from EMBED import int_to_bits;

def main(a: int) -> Array[bool, 32]:
    return int_to_bits(a)