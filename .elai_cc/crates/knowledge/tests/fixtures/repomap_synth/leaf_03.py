"""C-38 fixture: leaf caller 03. Imports mid_03_func only."""
from mid_03 import mid_03_func


def leaf_03_caller(x):
    return mid_03_func(x) - 3
