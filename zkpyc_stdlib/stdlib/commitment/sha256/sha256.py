from zk_types.types import Public, Array # zk_ignore
from hashes.sha256.sha256 import shaRound

# Initial values, FIPS 180-3, section 5.3.3
# https://csrc.nist.gov/csrc/media/publications/fips/180/3/archive/2008-10-31/documents/fips180-3_final.pdf
IV: Array[int, 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
]

# A commitment scheme based on sha256 that takes one u32[16] array as input,
# and returns the sha256 compression over the randomness as a u32[8].
# Note: no padding is applied
def commit(x: Array[int, 16], r: Array[int, 16]) -> Array[int, 8]:
	current: Array[int, 8] = IV
	current = shaRound(x, current)
	current = shaRound(r, current)
	return current