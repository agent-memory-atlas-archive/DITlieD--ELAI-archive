"""C-38 fixture: the central hub symbol imported by every caller_*.py
and mid_*.py module. PageRank will concentrate rank on `hub_root`
because it has the highest incoming-reference count in the fixture.
"""


def hub_root(value):
    """The single function whose top-1 PageRank ranking the contract
    test asserts. Implementation is trivial — the test only cares
    about the graph topology, not runtime behaviour.
    """
    return value + 1
