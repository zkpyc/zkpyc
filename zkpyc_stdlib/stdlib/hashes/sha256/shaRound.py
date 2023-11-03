from zk_types.types import Public, Array # zk_ignore

# FIPS 180-3, section 4.2.2
# https://csrc.nist.gov/csrc/media/publications/fips/180/3/archive/2008-10-31/documents/fips180-3_final.pdf
K: Array[int, 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
]

def rotr32(x: int, N: int) -> int:
    return (x >> N) | (x << (32 - N)) & 0xffffffff

def extend(w: Array[int, 64], i: int) -> int:
    s0: int = rotr32(w[i-15], 7) ^ rotr32(w[i-15], 18) ^ (w[i-15] >> 3) & 0xffffffff
    s1: int = rotr32(w[i-2], 17) ^ rotr32(w[i-2], 19) ^ (w[i-2] >> 10) & 0xffffffff
    return w[i-16] + s0 + w[i-7] + s1 & 0xffffffff

def temp1(e: int, f: int, g: int, h: int, k: int, w: int) -> int:
    # ch := (e and f) xor ((not e) and g)
    ch: int = (e & f) ^ ((~e & 0xffffffff) & g)

    #S1 := (e rightrotate 6) xor (e rightrotate 11) xor (e rightrotate 25)
    S1: int = rotr32(e, 6) ^ rotr32(e, 11) ^ rotr32(e, 25)
    
    #temp1 := h + S1 + ch + k + w
    return (h + S1 + ch + k + w) & 0xffffffff

def temp2(a: int, b: int, c: int) -> int:
    # maj := (a and b) xor (a and c) xor (b and c)
    maj: int = (a & b) ^ (a & c) ^ (b & c)

    # S0 := (a rightrotate 2) xor (a rightrotate 13) xor (a rightrotate 22)
    S0: int = rotr32(a, 2) ^ rotr32(a, 13) ^ rotr32(a, 22)

    # temp2 := S0 + maj
    return (S0 + maj) & 0xffffffff

# A function that computes one round of the SHA256 compression function given an input and the current value of the hash
# this is used by other components however many times needed
def shaRound(input: Array[int, 16], current: Array[int, 8]) -> Array[int, 8]:
    h0: int = current[0]
    h1: int = current[1]
    h2: int = current[2]
    h3: int = current[3]
    h4: int = current[4]
    h5: int = current[5]
    h6: int = current[6]
    h7: int = current[7]

    w: Array[int, 64] = [*input, *[0 for _ in range(48)]]

    for i in range(16, 64):
        w[i] = extend(w, i)

    a: int = h0
    b: int = h1
    c: int = h2
    d: int = h3
    e: int = h4
    f: int = h5
    g: int = h6
    h: int = h7

    for i in range(0, 64):
        t1: int = temp1(e, f, g, h, K[i], w[i])
        t2: int = temp2(a, b, c)

        h = g
        g = f
        f = e
        e = d + t1 & 0xffffffff
        d = c
        c = b
        b = a
        a = t1 + t2 & 0xffffffff

    h0 = h0 + a & 0xffffffff
    h1 = h1 + b & 0xffffffff
    h2 = h2 + c & 0xffffffff
    h3 = h3 + d & 0xffffffff
    h4 = h4 + e & 0xffffffff
    h5 = h5 + f & 0xffffffff
    h6 = h6 + g & 0xffffffff
    h7 = h7 + h & 0xffffffff

    return [h0, h1, h2, h3, h4, h5, h6, h7]
    