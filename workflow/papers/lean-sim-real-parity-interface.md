Source: QuantConnect LEAN Algorithmic Trading Engine
Reliability: MEDIUM-HIGH (official open-source engine, Apache 2.0, actively maintained)
Files examined:
  github.com/QuantConnect/Lean/blob/master/Common/Interfaces/IAlgorithm.cs
  github.com/QuantConnect/Lean/blob/master/Common/Interfaces/IBrokerage.cs
  github.com/QuantConnect/Lean/blob/master/Engine/AlgorithmManager.cs
  github.com/QuantConnect/Lean/blob/master/Engine/Setup/BacktestingSetupHandler.cs
  github.com/QuantConnect/Lean/blob/master/Engine/Engine.cs
  github.com/QuantConnect/Lean/blob/master/Engine/DataFeeds/Synchronizer.cs
  github.com/QuantConnect/Lean/blob/master/Engine/DataFeeds/LiveSynchronizer.cs
  github.com/QuantConnect/Lean/blob/master/Engine/DataFeeds/LiveTradingDataFeed.cs
  github.com/QuantConnect/Lean/blob/master/Algorithm/QCAlgorithm.cs
  github.com/QuantConnect/Lean/blob/master/Brokerages/Backtesting/BacktestingBrokerage.cs
  github.com/QuantConnect/Lean/blob/master/Common/Orders/Fills/EquityFillModel.cs
  quantconnect.com/docs/v2/writing-algorithms/live-trading/reconciliation
  quantconnect.com/docs/v2/writing-algorithms/reality-modeling/trade-fills/key-concepts


--- SECTION 1: SINGLE-ENTRYPOINT PATTERN ---

Pattern name: adapter-swap at engine boundary
The strategy author implements ONE interface: IAlgorithm (namespace QuantConnect.Interfaces)
The strategy code exposes two lifecycle methods the engine calls:
  void Initialize()       -- declare subscriptions, set parameters, configure models
  void OnData(Slice slice) -- receive each time slice, emit orders

The engine (AlgorithmManager.Run) drives a single event loop:
  foreach (var timeSlice in Stream(algorithm, synchronizer, results, token))
    { ...update securities... algorithm.OnData(algorithm.CurrentSlice); }

The ENGINE injects the execution adapter (IBrokerage) at startup, before Initialize() runs.
The strategy code never instantiates a brokerage directly.

For BACKTEST:
  BacktestingSetupHandler.CreateBrokerage() returns: new BacktestingBrokerage(uninitializedAlgorithm)
  implements: IBrokerage (via Brokerage base class)
  fill model: EquityFillModel, FutureFillModel, etc. -- pluggable via security.FillModel
  data source: Synchronizer (implements ISynchronizer) streams historical time slices

For LIVE:
  LiveSetupHandler.CreateBrokerage() returns: the real broker adapter (e.g. InteractiveBrokersBrokerage)
  implements: IBrokerage (same interface)
  fill model: broker execution -- fill price and fee set by the brokerage API response
  data source: LiveSynchronizer (extends Synchronizer) streams real-time ticks, heartbeats every 1s

Engine.cs selects the data synchronizer:
  var synchronizer = _liveMode ? new LiveSynchronizer() : new Synchronizer();

This is the verified structural basis for the H-24 "single-entrypoint" rule:
  - ONE IAlgorithm class, ONE Initialize(), ONE OnData() per strategy
  - The engine swaps IBrokerage + ISynchronizer implementations
  - Strategy code is NOT the adapter; the engine is

Key interfaces (C# namespaces):
  IAlgorithm     -- QuantConnect.Interfaces -- strategy contract
  IBrokerage     -- QuantConnect.Interfaces -- execution adapter contract
  ISynchronizer  -- QuantConnect.Interfaces -- data streaming contract
  IDataFeed      -- QuantConnect.Interfaces -- raw data subscription contract (implemented by LiveTradingDataFeed)
  ISetupHandler  -- QuantConnect.Lean.Engine.Setup -- selects which IBrokerage to instantiate

IAlgorithm.LiveMode property (verified from QCAlgorithm.cs):
  public bool LiveMode { get; }  -- backed by private _liveMode field
  doc comment: "Intended for use where certain behaviors will be enabled while the algorithm
  is trading live: such as notification emails, or displaying runtime statistics."
  Set by engine via: void SetLiveMode(bool live) (IAlgorithm interface method)
  Also available: AlgorithmMode { get; } (enum: Backtesting | PaperTrading | LiveTrading)
                  DeploymentTarget { get; }


--- SECTION 2: MODE-DIVERGENT-BRANCH ANTIPATTERN ---

LiveMode flag EXISTS and is accessible inside strategy code.
It is also called AlgorithmMode in the newer API.

LEGITIMATE use (confirmed from official doc comment and LEAN design intent):
  Notification emails, runtime statistics display, UI-only behavior
  Example: if (self.LiveMode): self.Notify.Email("trade@example.com", ...)

DANGEROUS use (the antipattern, grounds H-24 AST ban):
  if (self.LiveMode): self.MarketOrder(...)    # different signal in live vs backtest
  if (self.AlgorithmMode == AlgorithmMode.LiveTrading): self.SetHoldings(...)

Why this breaks parity (structural argument, verified from LEAN architecture):
  The backtest runs a DIFFERENT code path than live when a mode-branch guards execution/signal logic.
  Consequence: the backtest no longer predicts live behavior -- they are running different strategies.
  The adapter-swap pattern (Section 1) only preserves parity when the algorithm code path is IDENTICAL.
  A mode-branch in order/signal logic voids the parity guarantee entirely.

AlgorithmManager.cs has ENGINE-INTERNAL mode branches (legitimate):
  var backtestMode = (job.Type == PacketType.BacktestNode);
  if (_liveMode) { ProcessVolatilityHistoryRequirements(...) }  -- engine warmup, not strategy logic
  split/dividend handling and consolidator timing also branch on liveMode internally
  These are engine behaviors, not visible to the strategy's Initialize/OnData methods.

IMPORTANT SCOPING for H-24:
  The AST ban on mode-branches applies ONLY to:
    - execution-path modules (order generation, signal emission, position sizing, risk gates)
    - any code that runs within OnData or any method called by OnData that affects orders
  The AST ban does NOT apply to:
    - logging, notification, UI, display logic
    - algorithm warmup skip logic (self.IsWarmingUp checks are analogous and are acceptable)
    - engine-internal code outside IAlgorithm scope
  A blanket ban across ALL code would break legitimate notification uses.
  The H-24 checker must scope to the call-graph reachable from order-emission functions.


--- SECTION 3: RECONCILIATION (DEFERRED PART B) ---

What LEAN reconciliation actually compares (verified from quantconnect.com/docs/v2):
  PRIMARY METRIC: equity curve overlay -- live equity curve vs out-of-sample (OOS) backtest equity curve
    run in parallel from the live deployment start date
  The OOS backtest runs simultaneously with live execution (not a pre-deployment backtest)

Documented sim-real gap sources (official, verbatim categories from docs):
  1. Data Issues: provider differences, look-ahead bias, discrete time steps, custom data timing,
     split adjustments, tick slice sizes (backtest: 1ms slices; live: up to 70ms slices), auction timing
  2. Modeling Errors: reality modeling inaccuracies, unmeasured market impact, fill price discrepancies
  3. Brokerage Factors: portfolio allocation constraints, parameter mismatches, non-deterministic restarts,
     existing positions at deploy time, broker-specific margin rules
  4. Third-Party Indicators: price source differences, timestamp differences
  5. Scheduled Event Timing: execution precision differences
  6. Real-Time Delays: data arrival latency affecting scheduled event logic

Fill model gap (verified from source + docs):
  Backtest fill models (EquityFillModel etc.): ASSUME COMPLETE FILLS for all orders
    Code comment: "Currently, all orders assume complete fills (fill.FillQuantity = order.Quantity)"
    TODO in source: "Add DepthLimited fill partial order quantities based on tick quantity / bar.Volume"
  Live fills: PARTIAL FILLS occur naturally, set by brokerage API
  Official doc: "In backtests, the pre-built fill models assume orders completely fill"

Market impact gap (official statement):
  "LEAN does not currently model market impact, thus, fill prices can be better during a backtest
  than in live trading." -- quantconnect.com docs

Slippage gap:
  Backtest: asset.SlippageModel.GetSlippageApproximation() -- approximation applied to simulated fill
  Live: actual market slippage embedded in broker fill price; slippage model irrelevant

Latency gap:
  Backtest: no network latency; order submitted and fill returned in same time slice
  Live: orders sent to brokerage API asynchronously; fill returned asynchronously, can span slices

DEFERRED TIER-2 NUMERIC RECONCILIATION (Part B grounding):
  What it would compare: equity curve delta (live PnL vs OOS backtest PnL over same interval)
  Secondary: per-fill price deviation (actual fill vs simulated fill on same instrument/time)
  Tertiary: position state drift (holdings at T in live vs holdings at T in OOS backtest)
  Min-live-sample requirement: NOT DOCUMENTED by QuantConnect officially
    Honest statement: a parity claim on near-zero live activity is statistically meaningless
    Minimum meaningful sample: at minimum N completed round-trips (suggest N>=30) and a time window
    covering at least one full market regime (cannot be quantified from LEAN docs alone)
    Any numeric parity claim on <30 fills or <10 trading days is ungrounded conjecture
  The OOS backtest parallel-run is the correct structural baseline (verified from docs)


--- SECTION 4: HONEST RESIDUE ---

What identical-code-path (structural) parity CANNOT guarantee:
  1. MARKET IMPACT: large orders move the market; backtest fill assumes infinite liquidity
     LEAN explicitly states this gap is unmodeled and unresolvable without a custom impact model
  2. PARTIAL FILLS: live orders regularly partially fill; backtest assumes complete fills
     A strategy's position sizing and risk logic may behave differently under partial fills
  3. EXECUTION LATENCY: live orders have network + broker processing latency (10ms to seconds)
     A strategy that places orders at bar close may fill at next-bar open in backtest but later in live
  4. TICK SLICE TIMING: backtest 1ms vs live up to 70ms -- high-frequency strategies diverge here
  5. BROKERAGE MARGIN RULES: live margin can differ from the NullBuyingPowerModel or LEAN's model
  6. DATA SOURCE DRIFT: historical data provider (QuantQuote) vs live data feed may show different prices
     Community measured divergence: +7.45% backtest vs -2.45% live over 2 months (forum, one case)
  7. BORROWING COSTS: short positions incur borrow fees in live not modeled in backtest by default

Conclusion for H-24:
  Part A (structural check: single-entrypoint, no mode-divergent execution branches) +
  Part C (replay-hash: same code path confirmed) are NECESSARY but NOT SUFFICIENT for parity
  They eliminate the structural failure mode (different code in two modes)
  They do NOT eliminate the fill/latency/impact gap
  Part B (numeric reconciliation: equity curve delta over N>=30 fills on live) is the DEFERRED
  completion that measures residual gap after structural parity is confirmed
  Without Part B, "parity" means only "same code ran" not "same results"


--- NOT VERIFIED FROM SOURCE (flag) ---

UNVERIFIED: The exact location of LiveSynchronizer vs Synchronizer swap in Engine.cs was
  confirmed via WebFetch summary ("var synchronizer = _liveMode ? new LiveSynchronizer() : new Synchronizer()")
  but could not be confirmed line-number verbatim (Engine.cs returned a summary, not raw source).
  High confidence from the pattern + both Synchronizer.cs and LiveSynchronizer.cs verified directly.

UNVERIFIED: IDataFeed interface raw file returned 404 from raw.githubusercontent.com.
  LiveTradingDataFeed.cs confirmed to implement IDataFeed via "public class LiveTradingDataFeed : FileSystemDataFeed"
  and FileSystemDataFeed is known to implement IDataFeed; this is a one-hop inference, not direct source read.

UNVERIFIED: SetLiveMode call site in Engine.cs. The QCAlgorithm.cs read confirmed the method exists on
  IAlgorithm (void SetLiveMode(bool live)) and that _liveMode defaults to false with AlgorithmMode.Backtesting.
  The engine call to SetLiveMode was not read verbatim from Engine.cs (file too large, summarized).

UNVERIFIED: minimum live sample requirement for numeric reconciliation. QuantConnect docs do not specify
  this. The N>=30 suggestion is from statistical best practice (ungrounded conjecture -- label it as such
  in any H-24 spec text until a primary source is cited).


--- PUBLIC API SURFACE (for H-24 contract generation) ---

IAlgorithm key members relevant to H-24:
  void Initialize()                                          -- strategy entry (single entrypoint)
  void OnData(Slice slice)                                   -- data event (single entrypoint)
  bool LiveMode { get; }                                     -- READ-ONLY to strategy; SET by engine
  AlgorithmMode AlgorithmMode { get; }                       -- Backtesting | PaperTrading | LiveTrading
  void SetLiveMode(bool live)                                -- ENGINE use only; strategy must not call
  SecurityPortfolioManager Portfolio { get; }                -- position state
  SecurityTransactionManager Transactions { get; }           -- order management

IBrokerage key members relevant to H-24:
  bool PlaceOrder(Order order)                               -- adapter method; engine routes here
  bool UpdateOrder(Order order)
  bool CancelOrder(Order order)
  List<Order> GetOpenOrders()
  List<Holding> GetAccountHoldings()

ISynchronizer key members:
  IEnumerable<TimeSlice> StreamData(CancellationToken cancellationToken)  -- main data loop

BacktestingBrokerage: public class BacktestingBrokerage : Brokerage (implements IBrokerage)
  Instantiated by: BacktestingSetupHandler.CreateBrokerage()
  Fill execution: delegates to security.FillModel.Fill(context) per order

LiveTradingDataFeed: public class LiveTradingDataFeed : FileSystemDataFeed (implements IDataFeed)
  Adds: IDataQueueHandler _dataQueueHandler for real-time market data streaming
  Differs from backtest: concatenates file warmup + history + live stream; real-time price scaling


--- BEHAVIORAL CONTRACTS ---

BacktestingSetupHandler.CreateBrokerage -> returns BacktestingBrokerage (never live broker)
LiveSetupHandler (not read directly) -> returns real IBrokerage implementation
AlgorithmManager.Run -> drives ONE event loop over ISynchronizer.StreamData; calls algorithm.OnData
Engine.cs -> selects synchronizer class at startup based on _liveMode; never changes mid-run
IAlgorithm.LiveMode -> read-only from strategy perspective; set once at engine init; immutable during run
EquityFillModel.Fill -> assumes complete fill (fill.FillQuantity = order.Quantity); no partial fill model
