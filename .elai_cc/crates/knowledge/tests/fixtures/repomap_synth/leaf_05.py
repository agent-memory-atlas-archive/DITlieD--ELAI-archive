"""C-38 fixture: leaf caller 05. Imports mid_05_func only."""
from mid_05 import mid_05_func


def leaf_05_caller(x):
    return mid_05_func(x) - 5
