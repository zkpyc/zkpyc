from zk_types.types import Private, Array, field # zk_ignore
from mm import main as mm_multiply
from point import Pt

shear_map: Array[Array[field, 2], 2] = [
    [field(1), field(10)],
    [field(0), field(1)]
]

def main(pt: Private[Pt]) -> Pt:
    x: field = field(pt.x)
    y: field = field(pt.y)
    pt_matrix: Array[Array[field, 2], 2] = [[x, field(0)], [y, field(0)]]
    result: Array[Array[field, 2], 2] = mm_multiply(shear_map, pt_matrix)
    new_x: int = int(result[0][0])
    new_y: int = int(result[1][0])
    return Pt(x=new_x, y=new_y)
