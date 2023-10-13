from zk_types.types import Private, Array, field #zk_ignore

def main(x: Private[field]) -> Array[field, 6]:
    AB: Array[field, 2] = [x for _ in range(2)]
    CD: Array[field, 3] = [field(9), field(8), field(7)]
    new_arr: Array[field, 6] = [*AB, *CD, field(0)]
    return new_arr
