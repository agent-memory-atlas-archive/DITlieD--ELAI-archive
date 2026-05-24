"""C-38 fixture: leaf caller 08. Imports mid_08_func only."""
from mid_08 import mid_08_func


def leaf_08_caller(x):
    return mid_08_func(x) - 8
