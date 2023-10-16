from zk_types.types import Private, Array, field #zk_ignore
from dataclasses import dataclass #zk_ignore

@dataclass
class Pt:
    x: field
    y: field

@dataclass
class Pts:
    pts: Array[Pt, 2]

def main(y: Private[field]) -> field:
    p1: Pt = Pt(x=field(2), y=y)
    p2: Pt = Pt(x=y, y=field(2))
    pts: Array[Pts, 1] = [Pts(pts=[p1, p2])]
    return pts[0].pts[0].y * pts[0].pts[1].x
