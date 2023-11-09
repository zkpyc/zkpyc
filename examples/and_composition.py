from zk_types.types import Private, Public, Array # zk_ignore
from commitment.sha256.sha256 import commit
from ._3_plus_int import main as _3_plus_int

# We are going to verifiably compute _3_plus_int(x) and
# compose that with a proof for the pre-image of sha256(x)
def main(x: Private[int], rand: Private[Array[int, 16]], x_comm: Public[Array[int, 8]]) -> int:
    x_packed: Array[int, 16] = [x, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    assert(x_comm == commit(x_packed, rand))
    return _3_plus_int(x)
