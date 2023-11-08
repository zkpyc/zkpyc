from zk_types.types import Private, Array, field # zk_ignore
from utils.casts.int_from_bits import main as from_bits

def main(inputs: Private[Array[int, 16]]) -> Array[bool, 512]:
	out: int = from_bits(inputs)
	return out