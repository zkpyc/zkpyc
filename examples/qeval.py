from zk_types.types import Private, field # zk_ignore

def main(x: Private[field]) -> Private[field]:
    y: field = x**3
    return x + y + field(5)