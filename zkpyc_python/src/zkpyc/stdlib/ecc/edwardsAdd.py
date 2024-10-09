from zkpyc.types import Array, field # zk_ignore
from zkpyc.stdlib.ecc.edwardsParams import EdwardsParams


# Add two points on a twisted Edwards curve
# Curve parameters are defined with the last argument
# https://en.wikipedia.org/wiki/Twisted_Edwards_curve#Addition_on_twisted_Edwards_curves
def add(pt1: Array[field, 2], pt2: Array[field, 2], params: EdwardsParams) -> Array[field, 2]:

    a: field = params.EDWARDS_A
    d: field = params.EDWARDS_D

    u1: field = pt1[0]
    v1: field = pt1[1]
    u2: field = pt2[0]
    v2: field = pt2[1]

    uOut: field = (u1*v2 + v1*u2) / (field(1) + d*u1*u2*v1*v2)
    vOut: field = (v1*v2 - a*u1*u2) / (field(1) - d*u1*u2*v1*v2)

    return [uOut, vOut]