ELAI mockup — functional inventory map
source of truth: docs/thefinalglance/ELAI-FEATURE-MAP.md sections 1.1-1.13 (read fully),
  phase-index-MASTER.txt (sampled lines 1-616). B..K plan summaries NOT separately read —
  the feature map already absorbs them into status flags (R-7 safe-boundary stop).
format: AREA -> one-line purpose -> where it landed in elai-mockup.html

== BOOT / PROJECT SELECTION (brand surface) ==
project selection            pick or create a project to enter the dashboard      -> #boot recent-projects list
LOCAL/CLOUD/HYBRID mode      inference posture chosen at boot                      -> #boot mode row (3 segments)
recent projects              skeleton list of prior projects                      -> #boot recent list (3 ghost rows)
new project                  ghost CTA to start fresh                              -> #boot New Project button
stroke logo + draw anim      brand hero, per-stroke draw (verbatim)               -> #boot logo, click-to-replay

== DASHBOARD SHELL (work surface) ==
4 question-tabs              PROGRESS / PROJECT / APP / KNOWLEDGE primary nav      -> tab bar
running fraction bar         n/m segmented mission-control RUNNING meter           -> tab bar right (segbar + frac)
chat pane                    first-class chat alongside canvas                    -> .chat (message list + input + slash hint + usage meter)
canvas/chat split            draggable-looking divider                            -> .splitter between .chat and .canvas
statusbar                    FSM chip + session id + note (22px)                  -> footer .statusbar

== PROGRESS TAB (FSM / mission readiness) ==
mission + fraction           what is being built, n/m steps                       -> PROGRESS summary col
gates line                   quinary gate tally (ok / warn / fail)                -> PROGRESS summary GATES
last activity                executor/model/tokens live line                      -> PROGRESS summary LAST ACTIVITY
quinary legend               done/active/pending/warn/fail key                    -> PROGRESS summary LEGEND
FSM node graph               flat node graph, all 5 states demonstrated           -> PROGRESS .graph (6 nodes + capsule)
done capsule                 collapsed finished cluster (done by absence)         -> PROGRESS capsule
live edge                    marching dashed edge on the active node              -> PROGRESS edge.live
FSM full (9 states)          IDLE..CONFIGURING engine state                       -> statusbar chip + PROGRESS active node
agent system / BSP           executor/auditor/etc roles, supersteps               -> node chips (EXECUTOR sonnet), capsule
feasibility probe            pre-execution resource check                         -> represented as a pending node label
self-modification / DGM      promotion/fitness loop                               -> flyout monitors row (skeleton)

== PROJECT TAB (code intelligence) ==
code-graph (DKG/PageRank)    symbol graph + repo map                              -> PROJECT graph placeholder
lens bar                     Structure/Security/History/Cost/Dead-code views      -> PROJECT lens bar (5 lenses)
impact / blast-radius        reverse-dep cone, anti-regression gate               -> PROJECT detail panel skeleton row
dead-code islands            unreachable symbol detection                          -> PROJECT lens (Dead-code) + graph note
security lens                vibegate / taint / CWE findings                       -> PROJECT lens (Security)
cost lens                    $/resolved-task, token ledger                        -> PROJECT lens (Cost)
history lens                 time-travel / rewind / receipts                      -> PROJECT lens (History)
detail side panel            selected-symbol detail                               -> PROJECT .detail skeleton
multi-language DKG           18 grammars                                          -> PROJECT graph note label

== APP TAB (product preview / design) ==
pages / preview grid         app pages + live preview                             -> APP page grid skeleton
before/after recast          design recast writeback (two-tier)                   -> APP before/after split
dev-preview proxy            data-oid tagged live app                             -> APP preview frame placeholder
design modes                 APP / WEBSITE design canvas modes                    -> APP mode chips
fullstack codegen            framework manifests / SaaS archetypes                -> APP detail skeleton row

== KNOWLEDGE TAB (memory / research / retrieval) ==
memory facts list            Tier-A code-grounded + Tier-B preference             -> KNOWLEDGE facts list (sample rows)
provenance / taint chips     where a fact came from + trust tier                  -> KNOWLEDGE fact chips
research orchestrator        web research fan-out + citations                     -> KNOWLEDGE research area skeleton
retrieval / RAG              bm25/vector/pagerank/catrag/arag cascade             -> KNOWLEDGE retrieval status row
cosmos artifacts             cross-project artifact tier                          -> KNOWLEDGE detail skeleton row
infinite-context chat / memGPT digest memory                                     -> KNOWLEDGE memory hub label
staleness / intent-drift     stale-fact instrument                               -> KNOWLEDGE fact chip (STALE)

== FLYOUT (zero-width, slides over) ==
project intelligence         overview entry                                       -> flyout MENU
sessions / branches          session DAG, branch/merge                            -> flyout MENU
receipts / RRA               reproducible-run artifacts, attestation              -> flyout MENU
monitors                     the legacy panel long-tail, collapsed                -> flyout MONITORS group (collapsed skeleton sections)
settings                     opens settings modal                                 -> flyout MENU (opens modal)
file explorer                project file tree                                    -> flyout FILES tree skeleton
  -- monitors long-tail (represented as collapsed skeleton rows) --
  cost cockpit, runtime/backends, SSB telemetry, trust/taint telemetry,
  context/usage meter, critic decisions, skill systems, roadmap/compliance,
  proposals/MSI (self-mod), notifications, plan-approval, mode-routing,
  mutation/blast-radius gates, reviewer cockpit/DSR, activity orbit (retiring)
                                                                                  -> flyout MONITORS skeleton list

== SETTINGS MODAL (popup over workspace) ==
models / routing             per-role runtime, calibrated routing, tier matrix    -> settings nav + panel
sidecar                      local model boot/health, quantization                -> settings nav
gates                        wiring/mutation/blast-radius/trust gate config       -> settings nav
gating / authz               MCP authz, destructive-verb allowlist, hooks         -> settings nav
skills                       skill library, import policy, trust tiers            -> settings nav
cloud                        cloud mode, cost ceiling, BYOK/providers             -> settings nav
runtime                      backends, kernels, determinism                       -> settings nav
UX                           notifications, theme, density                        -> settings nav
advanced                     power-user inspectors (DST, attestation, eval, etc)  -> settings nav (Advanced group)

== CLI SURFACE (represented, not a UI room) ==
slash-command REPL           /task /research /plan /reason /ask etc               -> chat slash-command hint line
elai measure / doctor / wiring audit / rra show                                  -> noted in flyout receipts + statusbar

== NOT SURFACED AS ROOMS (engine internals, no user UI) ==
SSB single-writer, GRC state machine, context budget allocator, rules engine,
entropy quarantine, DST simulation, taint threading, IPC isolation — these are
substrate; they surface only via the telemetry monitors + gate verdicts above.
