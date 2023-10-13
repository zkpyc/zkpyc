from zk_types.types import Private, Array, field # zk_ignore
from dataclasses import dataclass # zk_ignore

import foo as bar

@dataclass
class Foo:
    foo: Array[field, 2]
    bar: Bar

def main(a: Private[Array[field, Q]]) -> Array[bool, 234 + 6]:
    a: field = field(1)
    a[32 + x][55] = foo(y)
    for i in range(0,3):
            assert(a == 1 + 2 + 3+ 4+ 5+ 6+ 6+ 7+ 8 + 4+ 5+ 3+ 4+ 2+ 3)
    assert(a.member == 1)
    return a