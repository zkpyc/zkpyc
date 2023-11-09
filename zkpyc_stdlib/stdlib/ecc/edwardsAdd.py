from zk_types.types import Array, field # zk_ignore
# from ecc.babyjubjubParams import BabyJubJubParams
from ecc.jubjubParams import BabyJubJubParams


# Add two points on a twisted Edwards curve
# Curve parameters are defined with the last argument
# https://en.wikipedia.org/wiki/Twisted_Edwards_curve#Addition_on_twisted_Edwards_curves
def main(pt1: Array[field, 2], pt2: Array[field, 2], context: BabyJubJubParams) -> Array[field, 2]:

    a: field = context.JUBJUB_A
    d: field = context.JUBJUB_D

    u1: field = pt1[0]
    v1: field = pt1[1]
    u2: field = pt2[0]
    v2: field = pt2[1]

    uOut: field = (u1*v2 + v1*u2) / (field(1) + d*u1*u2*v1*v2)
    vOut: field = (v1*v2 - a*u1*u2) / (field(1) - d*u1*u2*v1*v2)

    return [uOut, vOut]