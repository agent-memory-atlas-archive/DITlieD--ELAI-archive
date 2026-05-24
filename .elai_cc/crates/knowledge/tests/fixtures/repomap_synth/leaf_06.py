"""C-38 fixture: leaf caller 06. Imports mid_06_func only."""
from mid_06 import mid_06_func


def leaf_06_caller(x):
    return mid_06_func(x) - 6
