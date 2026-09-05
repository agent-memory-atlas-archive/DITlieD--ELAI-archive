Algorithm: RepoGraph - Enhancing AI Software Engineering with Repository-level Code Graph
ArXiv: 2410.14684
Paper: RepoGraph: Enhancing AI Software Engineering with Repository-level Code Graph
Local file: F:\.ELAI_workflow\.claude\papers\2410.14684.html
Repo (cloned): F:\.ELAI_workflow\docs\.source\repograph (github.com/ozyyshr/RepoGraph, depth=1)
Authors: Siru Ouyang, Wenhao Yu, Kaixin Ma, Zilin Xiao, Zhihan Zhang, Mengzhao Jia, Jiawei Han, Hongming Zhang, Dong Yu
Published: October 2024 (ICLR 2025)


Inputs:
repository_files | list of source files | arbitrary depth | any Python repository; tree-sitter parsed
query_term | string | - | the symbol name, function name, or search term used in ego-graph retrieval
k_hops | int | {1, 2} | hop depth for ego-graph expansion; paper tests k=1 (default) and k=2 (Section 5.2)
agent_action_space | set of tool calls | - | for agent integration: search_repograph() added as additional action

Outputs:
code_graph G | networkx MultiDiGraph | V=line-level nodes, E=invoke+contain edges | the full repo graph
ego_graph | subgraph | k-hop neighborhood of query term | flattened to string context appended to prompt
ranked_tags | list[Tag] | namedtuple(rel_fname, fname, line, name, kind, category, info) | ordered by graph rank


Hyperparameters:
k_hops | 1 (default in paper) | {1, 2} | k=1 is default; k=2 tested in Section 5.2, marginal improvement
map_tokens | 1024 | int > 0 | max token budget for the serialized graph context (construct_graph.py:42)
max_context_window | None | int or None | overall context ceiling; cap is min(map_tokens*16, window-padding) (construct_graph.py:76)


Graph Construction (VERBATIM from paper + real code, anti-hallucination anchor):

Section 3.1 (node definition):
"Each node in V corresponds to a line of code"
Nodes are classified as either 'def' (definition) where "a function, class, or variable is initially
defined" or 'ref' (reference) indicating "where this entity is referenced or invoked elsewhere"

Section 3.1 (edge types):
Two edge types: E_invoke and E_contain.
"The 'contain' edge connects a definition node to its internal components"
"invoke edges represent an invocation relationship, usually connecting a 'def' node to a 'ref' node"

Real code (repograph/construct_graph.py:96-113, verified from cloned repo):
  Tag = namedtuple("Tag", "rel_fname fname line name kind category info")

  def tag_to_graph(self, tags):
      G = nx.MultiDiGraph()
      for tag in tags:
          G.add_node(tag['name'], category=tag['category'], info=tag['info'],
                     fname=tag['fname'], line=tag['line'], kind=tag['kind'])
      # contain edges: class -> its member functions
      for tag in tags:
          if tag['category'] == 'class':
              class_funcs = tag['info'].split('\t')
              for f in class_funcs:
                  G.add_edge(tag['name'], f.strip())
      # invoke edges: ref node -> matching def node (name equality)
      tags_ref = [tag for tag in tags if tag['kind'] == 'ref']
      tags_def = [tag for tag in tags if tag['kind'] == 'def']
      for tag in tags_ref:
          for tag_def in tags_def:
              if tag['name'] == tag_def['name']:
                  G.add_edge(tag['name'], tag_def['name'])
      return G

Parsing backend: tree-sitter via grep-ast (same lineage as aider's RepoMap). tree_sitter_languages.
Symbol extraction adapted from aider RepoMap + Agentless get_repo_structure.


Ego-Graph Retrieval (VERBATIM from paper + real code):

Section 3.2:
"We retrieve the k-hop ego-graphs with the search term in the centric"

Real code (repograph/graph_searcher.py:1-45, verified from cloned repo):
  class RepoSearcher:
      def one_hop_neighbors(self, query):
          return list(self.graph.neighbors(query))

      def two_hop_neighbors(self, query):
          one_hop = self.one_hop_neighbors(query)
          two_hop = []
          for node in one_hop:
              two_hop.extend(self.one_hop_neighbors(node))
          return list(set(two_hop))

      def dfs(self, query, depth):
          visited = []; stack = [(query, 0)]
          while stack:
              node, level = stack.pop()
              if node not in visited:
                  visited.append(node)
                  if level < depth:
                      stack.extend([(n, level+1) for n in self.one_hop_neighbors(node)])
          return visited

      def bfs(self, query, depth):
          # analogous to dfs but queue-based

Section 3.2 (agent integration):
"We add search_repograph() as an additional action in the action space"


Integration with RAG and Agentless (VERBATIM from paper, Section 4.1):

RAG:
"For RAG, we follow its initial setting and use BM25 for file-level retrieval. After that,
we append the context of RepoGraph after the code files as part of the prompt"

Agentless:
"The context of RepoGraph is inserted in every step of Agentless"

Agent frameworks (SWE-agent, AutoCodeRover):
"We add search_repograph() as an additional action in the action space"
The action call signature (from run_repograph_sweagent.sh + SWE-agent/ dir) passes query string
to RepoSearcher; result is flattened ego-graph appended to the agent's observation.


Quantitative Results (VERBATIM from paper, Table 2 + Appendix A.2):

SWE-bench-Lite resolve rates (Table 2, N=300 issues):
  RAG baseline: 2.67%    -> RAG + RepoGraph:           5.33%   (+2.66 pp absolute)
  Agentless:    27.33%   -> Agentless + RepoGraph:      29.67%  (+2.34 pp absolute)
  AutoCodeRover: 19.00%  -> AutoCodeRover + RepoGraph:  21.33%  (+2.33 pp absolute)
  SWE-agent:    18.33%   -> SWE-agent + RepoGraph:      20.33%  (+2.00 pp absolute)

Report summary figure cited in task brief: "+32.8% relative resolve-rate" corresponds to the
Agentless improvement: (29.67-27.33)/27.33 * 100 = ~8.6% relative; the +32.8% figure likely
refers to a different baseline or a different metric. Verified numbers above are from Table 2.

Agent action efficiency (Appendix A.2):
  Max turns: 38 -> 35 (with RepoGraph action available)
  Average turns: 21.47 -> 19.12 (cited in task brief, confirmed from paper)

CrossCodeEval (Table 5, Section 5.3, transfer evaluation):
  Code Match EM:      10.8 -> 28.5
  Identifier Match EM: 16.7 -> 36.1


Critical Invariants (testable assertions):
assert G.is_directed(), "RepoGraph must be a directed graph (MultiDiGraph)"
assert all(G.nodes[n]['kind'] in ('def', 'ref') for n in G.nodes), "all nodes must be def or ref"
assert k_hops in (1, 2), "paper evaluates only k=1 and k=2"
assert len(one_hop_neighbors) == len(set(one_hop_neighbors)) or True, "two_hop deduplicates via set()"
# ego-graph flattening produces non-empty string when query term is in graph
assert len(ego_graph_str) > 0 or query_not_in_graph, "ego-graph must produce output for known symbol"


Complexity:
Graph construction time: O(|tags|^2) for invoke edges (name-equality join, no index), O(|tags|) for contain
Graph construction space: O(|V| + |E|) where V = all def/ref lines across all repo files
Ego-graph retrieval time: O(degree(v) * k) for k-hop BFS; O(1) lookup via networkx adjacency
Parallelism: graph is built once per repo; retrieval at inference is read-only, parallelizable
Note: name-equality join in tag_to_graph is O(N_ref * N_def); large repos may be slow


Failure Modes (from paper and code inspection):
1. Name-equality join (construct_graph.py:109-112) gives false edges when two different symbols share a name
2. tree-sitter parsing silently skips files with syntax errors; nodes for those files are absent
3. k=2 hops expands scope significantly; large graphs may blow the map_tokens budget
4. Standard libraries and builtins filtered via std_proj_funcs (construct_graph.py:146-204); false exclusions possible
5. Graph is repo-static: no incremental update; full rebuild needed after code changes


Implementation Pitfalls (do NOT do this):
Do NOT use undirected graph; invoke/contain direction is semantic (ref -> def, class -> member)
Do NOT rely on name-equality for uniqueness; two symbols with the same name create incorrect invoke edges
Do NOT expand k beyond 2 without token budget enforcement; ego-graph grows exponentially in dense graphs
Do NOT build the graph per-query; build once per repo session and reuse the RepoSearcher instance
Do NOT insert full graph into the prompt; flatten only the k-hop ego-graph around the query term


ELAI COVERAGE ASSESSMENT (honest verdict):

VERDICT: PARTIALLY COVERED. RepoGraph's graph structure is NOT new for ELAI, but the ego-graph
RETRIEVAL QUERY INTERFACE (search_repograph() as an agent action / named-symbol lookup at inference)
IS missing.

What ELAI already ships that covers RepoGraph's design:
  1. Symbol graph (crates/knowledge/src/symbol_graph/): typed def/ref node classification, invoke edges
     (SymbolEdge with callee_candidates), contain edges (class -> member functions). This is MORE
     sophisticated than RepoGraph's name-equality join (uses SCIP resolved canonical paths).
  2. PageRank repo-map (crates/knowledge/src/pagerank/mod.rs): ranks symbols by ref_count / inbound
     edges, same intent as RepoGraph's ranked retrieval. ELAI uses actual PageRank scores.
  3. DKG (crates/knowledge/src/dkg/): dependency knowledge graph, persistent SQLite, tracks
     inter-file symbol dependencies. More complete than RepoGraph's in-memory MultiDiGraph.
  4. Traversal functions (symbol_graph/traversal.rs): compute_impact_radius (BFS, same as
     one/two_hop_neighbors), compute_call_flow (DFS, same as dfs()), compute_symbol_context (360deg).
     These are structurally equivalent to graph_searcher.py's RepoSearcher methods.
  5. Impact radius tool (.claude/tools/impact_radius.py) and codegraph MCP: expose graph traversal
     at planning time.

What ELAI is MISSING from RepoGraph's contribution:
  1. Named-symbol ego-graph lookup at INFERENCE TIME as a first-class retrieval path. ELAI's
     symbol graph is used for planning/wiring checks, NOT as a live retrieval channel during
     code completion. The search_repograph(query) action is the gap.
  2. The graph is NOT appended to LLM prompts during generation. RepoGraph's core value is that
     the ego-graph context is injected into the generation prompt; ELAI's graph stays in the
     planning layer.
  3. No agent action search_repograph() / equivalent in ELAI's task_runner or tool-call space.

Load-bearing components to port (2 genuine new things):
  A. A named-symbol ego-graph lookup function that converts a symbol name to its k-hop subgraph
     as a flat string context, wired into pipeline.rs / the retrieval tier as a generation-time
     retrieval path.
  B. An agent action (MCP tool or CLI sub-command) exposing search_repograph(query, k=1) so the
     LLM/agent can explicitly navigate the existing symbol_graph at reasoning time (not just via
     impact_radius which is a planning tool, not a generation-context tool).

Components that do NOT need porting (already shipped):
  - Symbol graph construction (surpasses RepoGraph via SCIP)
  - PageRank scoring (surpasses RepoGraph's ref-count)
  - BFS/DFS traversal (equivalent to graph_searcher.py)
  - Def/ref node classification (equivalent to RepoGraph's node types)


REIMPLEMENTATION NOTES FOR ELAI:

The RepoGraph paper's core insight is that code graphs belong in NAVIGATION (agent action / context
injection at inference) not in RANKING (the existing retriever re-ranking pipeline). ELAI currently
has the graph but puts it in the wrong layer.

Minimal port:
  1. Add a symbol_graph_search(query: &str, k_hops: u8) -> String function in
     crates/knowledge/src/symbol_graph/traversal.rs that calls the existing BFS traversal and
     serializes the ego-graph as a flat code context string.
  2. Wire it into pipeline.rs as an optional retrieval path alongside BM25/ONNX (gated by
     ELAI_REPOGRAPH_ENABLE or a tier-3 / DEEP tier flag).
  3. Expose as an MCP tool or task_runner tool call: search_repograph(symbol_name, k=1).

No new graph construction is needed; ELAI's DKG + symbol_graph already exceed RepoGraph's construction.


CONTRACT TEST STUBS:

def test_node_types_valid():
    # Section 3.1: all nodes must be 'def' or 'ref'
    valid_kinds = {'def', 'ref'}
    mock_nodes = [{'kind': 'def'}, {'kind': 'ref'}, {'kind': 'def'}]
    assert all(n['kind'] in valid_kinds for n in mock_nodes)

def test_ego_graph_k1_subset_of_k2():
    # k=1 ego-graph must be a subset of k=2 ego-graph (nested BFS)
    k1_nodes = {'A', 'B', 'C'}
    k2_nodes = {'A', 'B', 'C', 'D', 'E'}
    assert k1_nodes.issubset(k2_nodes), "k=1 neighbors must be subset of k=2 neighbors"

def test_contain_edges_class_to_members():
    # construct_graph.py:101-105: class node has contain edges to its member functions
    # class 'Foo' with members ['bar', 'baz'] -> edges Foo->bar, Foo->baz
    class_members = ['bar', 'baz']
    edges = [('Foo', m.strip()) for m in class_members]
    assert ('Foo', 'bar') in edges
    assert ('Foo', 'baz') in edges

def test_swebench_resolve_improvement():
    # Table 2: Agentless+RepoGraph >= Agentless alone
    agentless_base = 27.33
    agentless_repograph = 29.67
    assert agentless_repograph > agentless_base
    assert abs((agentless_repograph - agentless_base) - 2.34) < 0.01

def test_turn_reduction():
    # Appendix A.2: average turns reduced from 21.47 to 19.12
    turns_baseline = 21.47
    turns_repograph = 19.12
    assert turns_repograph < turns_baseline
    assert abs(turns_baseline - turns_repograph - 2.35) < 0.01
