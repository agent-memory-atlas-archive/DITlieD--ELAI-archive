"""C-38 fixture: caller module 11. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_11_local(x):
    return hub_root(x) * 12
