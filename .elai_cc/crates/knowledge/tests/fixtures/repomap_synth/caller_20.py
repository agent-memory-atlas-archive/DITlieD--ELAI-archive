"""C-38 fixture: caller module 20. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_20_local(x):
    return hub_root(x) * 21
