from zk_types.types import Private, Array, field #zk_ignore
from dataclasses import dataclass #zk_ignore

@dataclass
class Pt:
    x: field
    y: field

@dataclass
class Pts:
    pts: Array[Pt, 2]

def main(pts: Private[Array[Pts, 1]]) -> field:
    return pts[0].pts[0].y * pts[0].pts[1].x
