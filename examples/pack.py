from zk_types.types import Private, field # zk_ignore
from zkpyc.stdlib.EMBED import get_field_size
from zkpyc.stdlib.EMBED import unpack
from zkpyc.stdlib.EMBED import pack

def main(x: Private[field]) -> field:
    return pack(unpack(x, get_field_size()))
