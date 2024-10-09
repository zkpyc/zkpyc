# https://eprint.iacr.org/2019/458.pdf

from zkpyc.types import Array, field # zk_ignore
from .constants import POSEIDON_C, POSEIDON_M

def ark(state: Array[field, 7], c: Array[field, 497], it: int) -> Array[field, 7]:
    out: Array[field, 7] = [*state]
    for i in range(0, 7):
        out[i] = out[i] + c[it + i]
    return out

def sbox(state: Array[field, 7], f: int, p: int, r: int) -> Array[field, 7]:
    out: Array[field, 7] = [*state]
    out[0] = out[0]**5
    for i in range(1, 7):
        out[i] = out[i]**5 if ((r < f/2) or (r >= f/2 + p)) else out[i]
    return out

def mix(state: Array[field, 7], m: Array[Array[field, 7], 7]) -> Array[field, 7]:
    out: Array[field, 7] = [field(0) for _ in range(7)]
    for i in range(0, 7):
        acc: field = field(0)
        for j in range(0, 7):
            acc = acc + (state[j] * m[i][j])
        out[i] = acc
    return out

# let N = 6 for now
def poseidon(inputs: Array[field, 6]) -> field:
    # assert(N > 0 && N <= 6); // max 6 inputs

    # t: int = 6 + 1
    rounds_p: Array[int, 8] = [56, 57, 56, 60, 60, 63, 64, 63]

    f: int = 8
    p: int = rounds_p[(7 - 2)]

    # Constants are padded with zeroes to the maximum value calculated by
    # t * (f + p) = 497, where `t` (number of inputs + 1) is a max of 7.
    # This is done to keep the function generic, as resulting array size depends on `t`
    # and we do not want callers passing down constants.
    # This should be revisited once compiler limitations are gone.

    c: Array[field, 497] = POSEIDON_C[7 - 2]
    m: Array[Array[field, 7], 7] = POSEIDON_M[7 - 2]

    state: Array[field, 7] = [field(0) for _ in range(7)]
    for i in range(1, 7):
        state[i] = inputs[i - 1]

    for r in range(0, f + p):
        state = ark(state, c, r * 7)
        state = sbox(state, f, p, r)
        state = mix(state, m)

    return state[0]