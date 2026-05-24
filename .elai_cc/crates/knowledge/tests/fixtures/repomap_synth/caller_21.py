"""C-38 fixture: caller module 21. Imports hub_root from hub_module."""
from hub_module import hub_root


def caller_21_local(x):
    return hub_root(x) * 22
