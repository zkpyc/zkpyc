from examples.zk_types.types import Private, Array, field # zk_ignore

def main(
    A: Private[Array[Array[field, 2], 2]],
    B: Private[Array[Array[field, 2], 2]]
) -> Array[Array[field, 2], 2]:
    AB: Array[Array[field, 2], 2] = [[field(0) for _ in range(2)] for _ in range(2)]
    for i in range(2):
        f_i: field = field(i)
        for j in range(2):
            f_j: field = field(j)
            for k in range(2):
                f_k: field = field(k)
                AB[f_i][f_j] = AB[f_i][f_j] + A[f_i][f_k] * B[f_k][f_j]
    return AB
