from zk_types.types import Array, field # zk_ignore
from dataclasses import dataclass #zk_ignore

# Parameters are based on: https://github.com/HarryR/ethsnarks/tree/9cdf0117c2e42c691e75b98979cb29b099eca998/src/jubjub
# Note: parameters will be updated soon to be more compatible with zCash's implementation

@dataclass
class BabyJubJubParams:
	JUBJUB_C: field
	JUBJUB_A: field
	JUBJUB_D: field
	MONT_A: field
	MONT_B: field
	INFINITY: Array[field, 2]
	Gu: field
	Gv: field

BABYJUBJUB_PARAMS: BabyJubJubParams = BabyJubJubParams(
    # Order of the curve for reference: 21888242871839275222246405745257275088614511777268538073601725287587578984328
    JUBJUB_C=field(8), # Cofactor
    JUBJUB_A=field(168700), # Coefficient A
    JUBJUB_D=field(168696), # Coefficient D

    # Montgomery parameters
    MONT_A=field(168698),
    MONT_B=field(1),

    # Point at infinity
    INFINITY=[field(0), field(1)],

    # Generator
    Gu=field(16540640123574156134436876038791482806971768689494387082833631921987005038935),
    Gv=field(20819045374670962167435360035096875258406992893633759881276124905556507972311)
)

def main() -> BabyJubJubParams:
    return BABYJUBJUB_PARAMS