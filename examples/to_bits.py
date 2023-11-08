from zk_types.types import Private, Array, field # zk_ignore
from utils.casts.int_to_bits import main as to_bits

def main(inputs: Private[Array[int, 16]]) -> Array[bool, 512]:
	out: Array[bool, 512] = [
		*to_bits(inputs[0]),
		*to_bits(inputs[1]),
		*to_bits(inputs[2]),
		*to_bits(inputs[3]),
		*to_bits(inputs[4]),
		*to_bits(inputs[5]),
		*to_bits(inputs[6]),
		*to_bits(inputs[7]),
		*to_bits(inputs[8]),
		*to_bits(inputs[9]),
		*to_bits(inputs[10]),
		*to_bits(inputs[11]),
		*to_bits(inputs[12]),
		*to_bits(inputs[13]),
		*to_bits(inputs[14]),
		*to_bits(inputs[15])
	]
	return out