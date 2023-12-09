from zk_types.types import Private, Array, field #zk_ignore
from dataclasses import dataclass #zk_ignore

@dataclass
class Pt:
    x: int
    y: int