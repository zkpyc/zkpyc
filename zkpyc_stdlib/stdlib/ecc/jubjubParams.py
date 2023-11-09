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

JUBJUB_PARAMS: BabyJubJubParams = BabyJubJubParams(
    # Order of the curve for reference: 52435875175126190479447740508185965837647370126978538250922873299137466033592
    JUBJUB_C=field(8), # Cofactor
    JUBJUB_A=field(52435875175126190479447740508185965837690552500527637822603658699938581184512), # Coefficient A
    JUBJUB_D=field(19257038036680949359750312669786877991949435402254120286184196891950884077233), # Coefficient D

    # Montgomery parameters
    MONT_A=field(40962),
    MONT_B=field(1),

    # Point at infinity
    INFINITY=[field(0), field(1)],

    # # Generator
    Gu=field(52355368488200756720908213129543630848976972731871436319321443845291207170897),
    Gv=field(18372611905088487385433946659983357101887954355879737496286092836680199584970)
)

def main() -> BabyJubJubParams:
    return JUBJUB_PARAMS