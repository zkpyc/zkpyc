from zk_types.types import Private, Array, field # zk_ignore
from utils.casts.int_from_bits import main as from_bits

def main(inputs: Private[Array[bool, 32]]) -> int:
	vals: Array[bool, 32] = inputs
	out: int = from_bits(vals)
	return out