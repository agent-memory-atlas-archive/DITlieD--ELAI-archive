"""C-38 fixture: leaf caller 00. Imports mid_00_func only."""
from mid_00 import mid_00_func


def leaf_00_caller(x):
    return mid_00_func(x) - 0
