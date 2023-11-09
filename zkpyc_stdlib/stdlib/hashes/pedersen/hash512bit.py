from zk_types.types import Array # zk_ignore
# from .hash512bitBool import main as pedersen
from .hash512bitBool_jubjub import main as pedersen
from utils.casts.int_to_bits import main as to_bits
from utils.casts.int_from_bits import main as from_bits

def main(inputs: Array[int, 16]) -> Array[int, 8]:
	e: Array[bool, 512] = [
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

	aC: Array[bool, 256] = pedersen(e)
	return [
		from_bits(aC[0:32]),
		from_bits(aC[32:64]),
		from_bits(aC[64:96]),
		from_bits(aC[96:128]),
		from_bits(aC[128:160]),
		from_bits(aC[160:192]),
		from_bits(aC[192:224]),
		from_bits(aC[224:256])
	]