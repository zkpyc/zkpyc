from zk_types.types import Private, field # zk_ignore

from arr_cls_arr_cls import Pt
from arr_cls_arr_cls import main as pt_mult

def mult(x: field, y: field) -> field:
    assert(x != y)
    return x * y

def main(x: Private[field], y: Private[field]) -> field:
    z: field = pt_mult(y)
    pt: Pt = Pt(x=x, y=z)
    return x * x if x == y else mult(pt.x, pt.y)
