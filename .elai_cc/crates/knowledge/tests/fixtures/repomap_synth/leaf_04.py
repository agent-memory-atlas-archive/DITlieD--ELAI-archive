"""C-38 fixture: leaf caller 04. Imports mid_04_func only."""
from mid_04 import mid_04_func


def leaf_04_caller(x):
    return mid_04_func(x) - 4
