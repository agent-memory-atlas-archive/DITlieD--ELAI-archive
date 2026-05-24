"""C-38 fixture: caller module 14. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_14_local(x):
    return hub_root(x) * 15
