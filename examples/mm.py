from zk_types.types import Private, Array, field # zk_ignore

def main(
    A: Private[Array[Array[field, 2], 2]],
    B: Private[Array[Array[field, 2], 2]]
) -> Array[Array[field, 2], 2]:
    AB: Array[Array[field, 2], 2] = [[field(0) for _ in range(2)] for _ in range(2)]
    for i in range(2):
        for j in range(2):
            for k in range(2):
                AB[i][j] = AB[i][j] + A[i][k] * B[k][j]
    return AB
