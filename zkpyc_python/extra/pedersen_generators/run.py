import sys
from ecc_params.config import set_curve_parameters
import numpy as np

def generate_params(curve_arg):
    if curve_arg == "bls12_381":
        c = 63
        # Obtain the JubJub group from the following parameters
        set_curve_parameters(
            52435875175126190479447740508185965837690552500527637822603658699938581184513,  # PARAM_Q
            52435875175126190479447740508185965837647370126978538250922873299137466033592,  # PARAM_E
            8,                                                                              # PARAM_C
            52435875175126190479447740508185965837690552500527637822603658699938581184512,  # PARAM_A
            19257038036680949359750312669786877991949435402254120286184196891950884077233   # PARAM_D
        )
    elif curve_arg == "bn256":
        c = 62
        # Obtain the BabyJubJub group from the following parameters
        set_curve_parameters(
            21888242871839275222246405745257275088548364400416034343698204186575808495617,  # PARAM_Q
            21888242871839275222246405745257275088614511777268538073601725287587578984328,  # PARAM_E
            8,                                                                              # PARAM_C
            168700,                                                                         # PARAM_A
            168696                                                                          # PARAM_D
        )
    elif curve_arg == "ristretto255":
        c = 62
        # Obtain the Doppio group from the following parameters
        set_curve_parameters(
            7237005577332262213973186563042994240857116359379907606001950938285454250989,   # PARAM_Q
            7237005577332262213973186563042994240793386170426921315009648928286698145284,   # PARAM_E
            4,                                                                              # PARAM_C
            1,                                                                              # PARAM_A
            7237005577332262213973186563042994240857116359379907606001950938285454187918    # PARAM_D
        )
    else:
        print("Invalid curve argument. Please use 'bls12_381', 'bn256', or 'ristretto255'.")
        sys.exit(1)

    from ecc_params.gadgets.pedersenHasher import PedersenHasher

    entropy = np.random.bytes(64)
    hasher_g = PedersenHasher("G")
    point_g = hasher_g.hash_bytes(entropy, c)
    print("# created from Point(x={}, y={})".format(point_g[0], point_g[1]))
    print(hasher_g.dsl_code)

    print("")

    entropy = np.random.bytes(64)
    hasher_h = PedersenHasher("H")
    point_h = hasher_h.hash_bytes(entropy, c)
    print("# created from Point(x={}, y={})".format(point_h[0], point_h[1]))
    print(hasher_h.dsl_code)

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python run.py <curve_arg>")
        sys.exit(1)

    curve_arg = sys.argv[1]
    generate_params(curve_arg)
