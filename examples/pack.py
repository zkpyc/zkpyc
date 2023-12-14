from zk_types.types import Private, field # zk_ignore
from EMBED import get_field_size
from EMBED import unpack
from EMBED import pack

def main(x: Private[field]) -> field:
    return pack(unpack(x, get_field_size()))
