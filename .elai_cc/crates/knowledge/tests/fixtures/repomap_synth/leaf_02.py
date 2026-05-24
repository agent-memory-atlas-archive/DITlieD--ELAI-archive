"""C-38 fixture: leaf caller 02. Imports mid_02_func only."""
from mid_02 import mid_02_func


def leaf_02_caller(x):
    return mid_02_func(x) - 2
