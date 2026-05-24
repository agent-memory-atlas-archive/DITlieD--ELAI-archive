"""C-38 fixture: leaf caller 09. Imports mid_09_func only."""
from mid_09 import mid_09_func


def leaf_09_caller(x):
    return mid_09_func(x) - 9
