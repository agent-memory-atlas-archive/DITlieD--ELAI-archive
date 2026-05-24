"""C-38 fixture: caller module 07. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_07_local(x):
    return hub_root(x) * 8
