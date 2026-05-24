"""C-38 fixture: caller module 00. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_00_local(x):
    return hub_root(x) * 1
