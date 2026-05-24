"""C-38 fixture: leaf caller 01. Imports mid_01_func only."""
from mid_01 import mid_01_func


def leaf_01_caller(x):
    return mid_01_func(x) - 1
