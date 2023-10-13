from typing import NewType, TypeVar, Generic

T = TypeVar('T')
N = TypeVar('N')

field = NewType('field', int)

class Public(Generic[T]):
    pass

class Private(Generic[T]):
    pass

class Array(Generic[T, N]):
    pass