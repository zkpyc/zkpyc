from zkpyc.types import Private, Array, field # zk_ignore
from zkpyc.stdlib.hashes.poseidon.poseidon import poseidon

# let N = 6 for now
def hash(inputs: Private[Array[field, 6]]) -> field:
    out: field = poseidon(inputs)
    return out