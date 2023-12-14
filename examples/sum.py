from zk_types.types import Private, Public, Array, field # zk_ignore
from EMBED import sum

class IntOrField:
    sum_field: field
    sum_int: int

def main(x: Private[Array[field, 5]], y: Private[Array[int,5]], _one: Public[field]) -> IntOrField:
    output: IntOrField = IntOrField(sum_field=sum(x)*_one, sum_int=sum(y))
    return output
