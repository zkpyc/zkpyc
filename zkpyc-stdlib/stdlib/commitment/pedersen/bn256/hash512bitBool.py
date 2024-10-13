from zkpyc.types import Array, field # zk_ignore
from zkpyc.stdlib.utils.multiplexer.lookup3bitSigned import sel3s
from zkpyc.stdlib.utils.multiplexer.lookup2bit import lookup as sel2
from zkpyc.stdlib.ecc.edwardsAdd import add
from zkpyc.stdlib.ecc.babyjubjubParams import BABYJUBJUB_PARAMS

def pedersen_no_compress(inputs: Array[bool, 512], generator: Array[Array[Array[field, 2], 4], 171]) -> Array[field, 2]:
    e: Array[bool, 513] = [
        *inputs,
        False
    ]

    a: Array[field, 2] = BABYJUBJUB_PARAMS.INFINITY # Infinity
    cx: field = field(0)
    cy: field = field(0)

    for i in range(0, 171):
        cx = sel3s([e[3*i], e[3*i + 1], e[3*i + 2]], [generator[i][0][0], generator[i][1][0], generator[i][2][0], generator[i][3][0]])
        cy = sel2([e[3*i], e[3*i + 1]], [generator[i][0][1], generator[i][1][1], generator[i][2][1], generator[i][3][1]])
        a = add(a, [cx, cy], BABYJUBJUB_PARAMS)

    return a