from zk_types.types import Private, Array, field # zk_ignore
from zkpyc.stdlib.ecc.edwardsAdd import add
from zkpyc.stdlib.ecc.babyjubjubParams import BABYJUBJUB_PARAMS

def main(pt1: Private[Array[field, 2]], pt2: Private[Array[field, 2]]) -> Array[field, 2]:
    out: Array[field, 2] = add(pt1, pt2, BABYJUBJUB_PARAMS)
    return out