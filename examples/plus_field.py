from zk_types.types import Private, Public, field # zk_ignore

def main(x: Private[field], y: Private[field], _one: Public[field]) -> field:
    out: field = (x + y) * _one
    return out