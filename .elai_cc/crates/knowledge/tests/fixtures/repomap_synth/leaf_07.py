"""C-38 fixture: leaf caller 07. Imports mid_07_func only."""
from mid_07 import mid_07_func


def leaf_07_caller(x):
    return mid_07_func(x) - 7
