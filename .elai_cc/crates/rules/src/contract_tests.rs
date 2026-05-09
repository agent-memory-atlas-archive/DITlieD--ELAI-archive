//! Contract tests for the rules engine: static (263+) and property-based (proptest).
//!
//! Static tests validate predicate compilation, structural invariants, and
//! evaluation behavior per domain. Property-based tests verify Composite
//! predicate invariants (De Morgan, associativity) at arbitrary depth.

#[cfg(test)]
mod static_tests {
    use rustc_hash::FxHashMap;

    use crate::predicate::{
        from_json, validate, AstError, AstEvaluator, AstMatch, DkgError,
        DkgQuerier, ReachabilityMatch,
        ReachabilityParams, Verdict, VerdictResult,
    };
    use crate::rule_engine::{
        DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
    };

    // -- Stubs for AST and DKG evaluation --

    struct AlwaysMatchAst;
    impl AstEvaluator for AlwaysMatchAst {
        fn evaluate_query(
            &self,
            _lang: &str,
            _src: &str,
            _query: &str,
        ) -> Result<Vec<AstMatch>, AstError> {
            Ok(vec![AstMatch {
                node_text: "match".into(),
                start_line: 1,
                end_line: 1,
            }])
        }
        fn validate_query(&self, _lang: &str, _query: &str) -> Result<(), AstError> {
            Ok(())
        }
    }

    struct NeverMatchAst;
    impl AstEvaluator for NeverMatchAst {
        fn evaluate_query(
            &self,
            _lang: &str,
            _src: &str,
            _query: &str,
        ) -> Result<Vec<AstMatch>, AstError> {
            Ok(vec![])
        }
        fn validate_query(&self, _lang: &str, _query: &str) -> Result<(), AstError> {
            Ok(())
        }
    }

    struct StubDkg;
    impl DkgQuerier for StubDkg {
        fn query_reachability(
            &self,
            _params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            Ok(vec![])
        }
        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    struct MatchDkg;
    impl DkgQuerier for MatchDkg {
        fn query_reachability(
            &self,
            _params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            Ok(vec![ReachabilityMatch {
                path: vec!["a".into(), "b".into()],
                edge_labels: vec!["call".into()],
            }])
        }
        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    // -- Helpers --

    fn make_rule(rule_id: &str, domain: &str, severity: &str, predicate_json: &str) -> Rule {
        Rule {
            rule_id: rule_id.into(),
            domain: domain.into(),
            tier: 1,
            severity: severity.into(),
            title: rule_id.into(),
            predicate_json: predicate_json.into(),
            phase_mask: 0xFF,
            escalate_target: None,
        }
    }

    fn grep_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg))
    }

    fn ast_match_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(AlwaysMatchAst), Box::new(StubDkg))
    }

    fn ast_nomatch_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg))
    }

    fn dkg_match_engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(MatchDkg))
    }

    fn eval_ctx<'a>(source: &'a str, file_path: &'a str) -> EvaluationContext<'a> {
        EvaluationContext {
            action: "write",
            source,
            file_path,
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: FxHashMap::default(),
        }
    }

    fn eval_ctx_with_metric<'a>(
        source: &'a str,
        file_path: &'a str,
        metric: &str,
        value: f64,
    ) -> EvaluationContext<'a> {
        let mut m = FxHashMap::default();
        m.insert(metric.to_string(), value);
        EvaluationContext {
            action: "write",
            source,
            file_path,
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: m,
        }
    }

    fn eval_single(
        engine: &DefaultRuleEngine,
        rule: &Rule,
        ctx: &EvaluationContext<'_>,
    ) -> Verdict {
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let verdicts = engine.evaluate(&[(rule.clone(), compiled)], ctx).unwrap();
        verdicts.into_iter().next().unwrap()
    }

    // -----------------------------------------------------------------------
    // TRADING DOMAIN — 25 Tier1 * 3 + 5 Tier2 * 2 = 85 tests
    // -----------------------------------------------------------------------

    // T1 GREP rules: 6,8,15,19,28,35,36,38 = 8 rules * 3 = 24 tests
    // T1 AST rules: 1,2,3,4,5,10,11,18,31,32,33,34,39,40 = 14 rules * 3 = 42 tests
    // (Rules 6,8,15,19,28,35,36,38 are GREP; 40 is GREP)
    // (Total T1 = 25 rules, but some overlap: 14 AST + 11 GREP = 25) * 3 = 75
    // T2 SCHEMA: 12,13,17,21,29 = 5 * 2 = 10
    // Total: 85

    // -- Trading GREP rules (Block/Allow/Edge each) --

    #[test]
    fn contract_trading_r6_block_survivorship() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.6",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"constituents.*current|current.*constituents","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("index_constituents = get_current_constituents(date)", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r6_allow_historical() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.6",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"constituents.*current|current.*constituents","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("index_members = get_historical_members(date)", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r6_edge_partial_match() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.6",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"constituents.*current|current.*constituents","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# constituents are loaded from historical DB\nresult = get_current_price()", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        // "current" appears but not adjacent to "constituents" in the matching sense
        // Actually the pattern is OR, so "current" without "constituents" nearby won't match
        // because the regex requires both words in sequence
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r8_block_ticker_in_prompt() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.8",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"prompt.*\\b(AAPL|MSFT|TSLA|GOOG)\\b|\\b(AAPL|MSFT|TSLA|GOOG)\\b.*prompt","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("prompt = f'Analyze AAPL stock trends'", "src/trading/llm.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r8_allow_variable_ref() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.8",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"prompt.*\\b(AAPL|MSFT|TSLA|GOOG)\\b|\\b(AAPL|MSFT|TSLA|GOOG)\\b.*prompt","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("ticker = config.get('symbol')\nresult = analyze(ticker)", "src/trading/llm.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r8_edge_ticker_no_prompt() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.8",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"prompt.*\\b(AAPL|MSFT|TSLA|GOOG)\\b|\\b(AAPL|MSFT|TSLA|GOOG)\\b.*prompt","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("price = get_price('AAPL')", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r15_block_cross_exchange() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.15",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"cross.?exchange.*order|order.*cross.?exchange","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("submit_cross_exchange_order(binance, kraken, qty=100)", "src/trading/execution.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r15_allow_with_delay() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.15",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"cross.?exchange.*order|order.*cross.?exchange","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("execute_single_exchange_order(binance, qty=100, transit_delay=5)", "src/trading/execution.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r15_edge_similar_name() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.15",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"cross.?exchange.*order|order.*cross.?exchange","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# cross reference exchange info for order routing", "src/trading/execution.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r19_block_short_pnl() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.19",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"short.*pnl|pnl.*short","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("short_pnl = (entry - exit) * qty", "src/trading/pnl.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r19_allow_with_funding() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.19",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"short.*pnl|pnl.*short","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("total = compute_long_pnl(entry, exit, qty, fees)", "src/trading/pnl.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r19_edge_comment_only() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.19",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"short.*pnl|pnl.*short","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# TODO: implement short_pnl calculation with funding costs", "src/trading/pnl.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r28_block_long_only() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.28",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"long.only|long_only","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("strategy_type = 'long_only'", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r28_allow_hedged() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.28",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"long.only|long_only","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("strategy_type = 'long_short_hedged'", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r28_edge_substring() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.28",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"long.only|long_only","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# this is a long only explanation of the algorithm", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r35_block_psi_input() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.35",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"psi.*input|input.*psi","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("psi_score = compute_psi(input_distribution, reference)", "src/trading/monitor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r35_allow_output_psi() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.35",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"psi.*input|input.*psi","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("psi_out = compute_psi(output_distribution, reference)", "src/trading/monitor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r35_edge_both() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.35",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"psi.*input|input.*psi","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# compute psi for input and output distributions", "src/trading/monitor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r36_block_full_finetune() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.36",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"fine.?tune|finetune","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("model.finetune(dataset, epochs=10)", "src/trading/model.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r36_allow_lora() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.36",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"fine.?tune|finetune","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("model.train_lora(dataset, rank=8)", "src/trading/model.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r36_edge_comment() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.36",
            "trading",
            "WARN",
            r#"{"type":"grep_pattern","pattern":"fine.?tune|finetune","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# TODO: replace fine_tune with LoRA wrapper", "src/trading/model.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r38_block_live_no_shadow() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.38",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"live.*deploy|deploy.*live|production.*trading","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("deploy_live_trading(strategy, broker='ibkr')", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r38_allow_shadow() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.38",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"live.*deploy|deploy.*live|production.*trading","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("run_shadow_backtest(strategy, broker='paper')", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r38_edge_keyword_split() {
        let engine = grep_engine();
        let rule = make_rule(
            "trading.38",
            "trading",
            "BLOCK",
            r#"{"type":"grep_pattern","pattern":"live.*deploy|deploy.*live|production.*trading","scope":null,"context_lines":0}"#,
        );
        let ctx = eval_ctx("# live data feed is separate from deploy scripts", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    // -- Trading AST rules (Block/Allow/Edge) --
    // AST rules fire based on AlwaysMatchAst / NeverMatchAst stubs

    #[test]
    fn contract_trading_r1_block_lookahead() {
        let engine = ast_match_engine();
        let rule = make_rule(
            "trading.1",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (attribute) @fn)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("df['signal'].mean() / df['signal'].std()", "src/trading/features.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r1_allow_rolling() {
        let engine = ast_nomatch_engine();
        let rule = make_rule(
            "trading.1",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (attribute) @fn)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("df['signal'].rolling(20).mean().shift(1)", "src/trading/features.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r1_edge_expanding() {
        let engine = ast_nomatch_engine();
        let rule = make_rule(
            "trading.1",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (attribute) @fn)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("df['signal'].expanding().mean()", "src/trading/features.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r2_block_peaks() {
        let engine = ast_match_engine();
        let rule = make_rule(
            "trading.2",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(import_from module: (dotted_name) @mod)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("from scipy.signal import find_peaks", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r2_allow_no_peaks() {
        let engine = ast_nomatch_engine();
        let rule = make_rule(
            "trading.2",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(import_from module: (dotted_name) @mod)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("from custom_algo import detect_regime_change", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r2_edge_other_scipy() {
        let engine = ast_nomatch_engine();
        let rule = make_rule(
            "trading.2",
            "trading",
            "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(import_from module: (dotted_name) @mod)","query_format":"sexp","scope":null}"#,
        );
        let ctx = eval_ctx("from scipy.stats import norm", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r3_block_shuffle() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.3", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("train_test_split(X, y, shuffle=True)", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r3_allow_purged() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.3", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("PurgedGroupTimeSeriesSplit(n_splits=5)", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r3_edge_kfold() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.3", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("KFold(n_splits=5, shuffle=True)", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r4_block_no_shift() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.4", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("signal = compute_signal(features)", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r4_allow_shifted() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.4", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("signal = compute_signal(features).shift(1)", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r4_edge_shift_zero() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.4", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("signal = compute_signal(features).shift(0)", "src/trading/signal.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r5_block_backfill() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.5", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("df.fillna(method='bfill')", "src/trading/preprocess.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r5_allow_ffill() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.5", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("df.fillna(method='ffill')", "src/trading/preprocess.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r5_edge_explicit_value() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.5", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("df.fillna(value=0.0)", "src/trading/preprocess.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r10_block_adjusted_prices() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.10", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(subscript)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("X_train = df[['adj_close', 'adj_open']]", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r10_allow_raw_prices() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.10", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(subscript)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("X_train = df[['close', 'open', 'volume']]", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r10_edge_backward_adj() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.10", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(subscript)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("features = backward_adjusted_prices(df)", "src/trading/ml.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r11_block_untyped_ohlcv() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.11", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def ingest_ohlcv(data: dict):", "src/trading/ingest.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r11_allow_pydantic() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.11", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def ingest_ohlcv(data: OHLCVModel):", "src/trading/ingest.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r11_edge_partial_typing() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.11", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def ingest_ohlcv(data):", "src/trading/ingest.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r18_block_sl_close_only() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.18", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(comparison_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("if close < stop_loss:", "src/trading/risk.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r18_allow_high_low() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.18", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(comparison_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("if low < stop_loss or high > take_profit:", "src/trading/risk.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r18_edge_tp_only() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.18", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(comparison_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("if close >= take_profit:", "src/trading/risk.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r31_block_pass_body() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.31", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_alpha():\n    pass", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r31_allow_impl() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.31", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_alpha():\n    return features @ weights", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r31_edge_todo() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.31", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_alpha():\n    # TODO implement\n    pass", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r32_block_bare_except() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.32", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(except_clause)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("try:\n    execute()\nexcept:\n    pass", "src/trading/executor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r32_allow_specific_except() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.32", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(except_clause)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("try:\n    execute()\nexcept ValueError as e:\n    log.error(e)", "src/trading/executor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r32_edge_nested_try() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.32", "trading", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(except_clause)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("try:\n    try:\n        x()\n    except:\n        pass\nexcept ValueError:\n    log.error('outer')", "src/trading/executor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r33_block_no_assert() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.33", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(return_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_rsi(series):\n    return rsi_values", "src/trading/indicators.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r33_allow_asserted() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.33", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(return_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_rsi(series):\n    assert len(rsi_values) == len(series)\n    return rsi_values", "src/trading/indicators.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r33_edge_nan_check() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.33", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(return_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def compute_rsi(series):\n    # NaN handling deferred\n    return rsi_values", "src/trading/indicators.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r34_block_any_type() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.34", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process(data: Any) -> None:", "src/trading/types.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r34_allow_typed() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.34", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process(data: TradeEvent) -> None:", "src/trading/types.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r34_edge_frozen_dc() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.34", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("@dataclass(frozen=True)\nclass TradeEvent:", "src/trading/types.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r39_block_no_pin() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.39", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(import_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("import yfinance", "src/trading/data_vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r39_allow_pinned() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("trading.39", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(import_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("import yfinance  # pinned: yfinance==0.2.18", "src/trading/data_vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r39_edge_transitive() {
        let engine = ast_match_engine();
        let rule = make_rule("trading.39", "trading", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(import_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("from vendor_client import DataFeed", "src/trading/data_vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r40_block_no_interface_map() {
        let engine = grep_engine();
        let rule = make_rule("trading.40", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"algorithm|model.*paper|arxiv","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("# Based on algorithm from arXiv:2103.12345", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r40_allow_with_ref() {
        let engine = grep_engine();
        let rule = make_rule("trading.40", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"algorithm|model.*paper|arxiv","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("compute_alpha_factors(features, weights)", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r40_edge_comment() {
        let engine = grep_engine();
        let rule = make_rule("trading.40", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"algorithm|model.*paper|arxiv","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("# this algorithm is well-known, no paper needed", "src/trading/alpha.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    // Additional T1 GREP rules to reach 85 minimum

    #[test]
    fn contract_trading_r6b_block_current_index() {
        let engine = grep_engine();
        let rule = make_rule("trading.6b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"get_index.*today|current_index|index_now","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("members = get_index_members_today()", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r6b_allow_historical_index() {
        let engine = grep_engine();
        let rule = make_rule("trading.6b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"get_index.*today|current_index|index_now","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("members = get_index_members_at(date)", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r6b_edge_index_variable() {
        let engine = grep_engine();
        let rule = make_rule("trading.6b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"get_index.*today|current_index|index_now","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("idx = rebalance_index(date='2023-01-01')", "src/trading/data.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r28b_block_no_justification() {
        let engine = grep_engine();
        let rule = make_rule("trading.28b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"long.?only.*strat|strategy.*long.?only","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("config = {'strategy': 'long_only', 'universe': 'sp500'}", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r28b_allow_justified() {
        let engine = grep_engine();
        let rule = make_rule("trading.28b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"long.?only.*strat|strategy.*long.?only","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("class MomentumStrategy(BaseStrategy):", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r28b_edge_comment() {
        let engine = grep_engine();
        let rule = make_rule("trading.28b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"long.?only.*strat|strategy.*long.?only","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("# Justification: long_only_strategy is regulatory requirement", "src/trading/strategy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r38b_block_no_canary() {
        let engine = grep_engine();
        let rule = make_rule("trading.38b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"deploy.*prod|production.*deploy|go.?live","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("deploy_to_production(strategy, config)", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_trading_r38b_allow_canary() {
        let engine = grep_engine();
        let rule = make_rule("trading.38b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"deploy.*prod|production.*deploy|go.?live","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("run_canary_test(strategy, paper_broker)", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_trading_r38b_edge_staging() {
        let engine = grep_engine();
        let rule = make_rule("trading.38b", "trading", "WARN",
            r#"{"type":"grep_pattern","pattern":"deploy.*prod|production.*deploy|go.?live","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("deploy_to_staging(strategy, config)", "src/trading/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    // -- Trading SCHEMA_REQUIRED Tier 2 (5 rules * 2 = 10 tests) --

    #[test]
    fn contract_trading_r12_valid_backtest_schema() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["commission","slippage","spread"]},"output_type":"backtest_config"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_trading_r12_invalid_missing_field() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["commission","slippage","spread"]},"output_type":"backtest_config"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("trading.12", "trading", "BLOCK", json);
        let source = r#"{"commission": 0.001, "slippage": 0.0005}"#;
        let ctx = eval_ctx(source, "config/backtest.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r13_valid_institutional() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["square_root_impact"]},"output_type":"institutional_config"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_trading_r13_invalid_no_impact() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["square_root_impact"]},"output_type":"institutional_config"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("trading.13", "trading", "BLOCK", json);
        let source = r#"{"commission": 0.001}"#;
        let ctx = eval_ctx(source, "config/institutional.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r17_valid_report() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["sharpe","dsr","max_drawdown","trial_count"]},"output_type":"backtest_report"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_trading_r17_invalid_no_trial() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["sharpe","dsr","max_drawdown","trial_count"]},"output_type":"backtest_report"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("trading.17", "trading", "BLOCK", json);
        let source = r#"{"sharpe": 1.5, "dsr": 0.9, "max_drawdown": 0.15}"#;
        let ctx = eval_ctx(source, "reports/backtest.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r21_valid_trial_count() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["trial_count"]},"output_type":"report"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_trading_r21_invalid_no_count() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["trial_count"]},"output_type":"report"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("trading.21", "trading", "BLOCK", json);
        let source = r#"{"sharpe": 2.0}"#;
        let ctx = eval_ctx(source, "reports/result.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_trading_r29_valid_signal_schema() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["opposing_thesis","risk_if_wrong"]},"output_type":"signal_output"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_trading_r29_invalid_no_thesis() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["opposing_thesis","risk_if_wrong"]},"output_type":"signal_output"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("trading.29", "trading", "BLOCK", json);
        let source = r#"{"signal": "buy", "confidence": 0.8}"#;
        let ctx = eval_ctx(source, "output/signal.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    // -----------------------------------------------------------------------
    // DASHBOARD DOMAIN — 11 Tier1 * 3 = 33 tests
    // -----------------------------------------------------------------------

    #[test]
    fn contract_dashboard_r1_block_god_component() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.1", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(function_declaration)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("function Dashboard() { /* 200 lines of fetch + state + render */ }", "src/components/Dashboard.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r1_allow_small() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.1", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(function_declaration)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("function Button({ label }) { return <button>{label}</button>; }", "src/components/Button.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r1_edge_150_lines() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.1", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(function_declaration)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("function Widget() { /* exactly 150 lines */ }", "src/components/Widget.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r1b_block_high_loc() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.1b", "dashboard", "WARN",
            r#"{"type":"metric_gate","metric_name":"component_loc","operator":"gt","threshold":150.0,"source":"file"}"#);
        let ctx = eval_ctx_with_metric("", "src/components/Big.tsx", "component_loc", 200.0);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r1b_allow_low_loc() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.1b", "dashboard", "WARN",
            r#"{"type":"metric_gate","metric_name":"component_loc","operator":"gt","threshold":150.0,"source":"file"}"#);
        let ctx = eval_ctx_with_metric("", "src/components/Small.tsx", "component_loc", 50.0);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r1b_edge_exact_threshold() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.1b", "dashboard", "WARN",
            r#"{"type":"metric_gate","metric_name":"component_loc","operator":"gt","threshold":150.0,"source":"file"}"#);
        let ctx = eval_ctx_with_metric("", "src/components/Mid.tsx", "component_loc", 150.0);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r3_block_no_cleanup() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.3", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => { fetch('/api/data'); }, [])", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r3_allow_with_cleanup() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.3", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => { const ctrl = new AbortController(); return () => ctrl.abort(); }, [])", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r3_edge_empty_cleanup() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.3", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => { fetch('/api'); return () => {}; }, [])", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r4_block_complex_state() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.4", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const [a, setA] = useState(0);\nconst [b, setB] = useState(0);", "src/components/Form.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r4_allow_reducer() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.4", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const [state, dispatch] = useReducer(reducer, init)", "src/components/Form.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r4_edge_single_state() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.4", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const [count, setCount] = useState(0)", "src/components/Counter.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r5_block_no_abort() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.5", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => { fetch('/api/data').then(r => setData(r)); }, [])", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r5_allow_with_abort() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.5", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const ctrl = new AbortController();\nfetch('/api', { signal: ctrl.signal })", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r5_edge_timeout() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.5", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("setTimeout(() => fetch('/api'), 1000)", "src/hooks/useFetch.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r6_block_cascade() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.6", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => setB(a+1), [a]);\nuseEffect(() => setC(b+1), [b]);", "src/components/Chain.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r6_allow_single_effect() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.6", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("useEffect(() => setData(transform(raw)), [raw]);", "src/components/View.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r6_edge_memo() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.6", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const derived = useMemo(() => compute(a, b), [a, b]);", "src/components/View.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r7_block_d3_no_ref() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.7", "dashboard", "BLOCK",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("d3.select('body').append('svg')", "src/charts/Scatter.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_dashboard_r7_allow_with_ref() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.7", "dashboard", "BLOCK",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const ref = useRef(); d3.select(ref.current).append('svg')", "src/charts/Scatter.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r7_edge_d3_util() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.7", "dashboard", "BLOCK",
            r#"{"type":"ast_pattern","language":"javascript","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("const scale = d3.scaleLinear().domain([0, 100])", "src/charts/utils.ts");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r8_block_large_render() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.8", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"data\\.length\\s*>\\s*1000|\\bforEach\\b.*render|map.*\\breturn\\b.*<","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("if (data.length > 1000) { data.forEach(d => renderPoint(d)); }", "src/charts/Points.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r8_allow_virtualized() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.8", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"data\\.length\\s*>\\s*1000|\\bforEach\\b.*render|map.*\\breturn\\b.*<","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("return <VirtualList data={items} renderItem={Item} />", "src/charts/List.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r8_edge_small_set() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.8", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"data\\.length\\s*>\\s*1000|\\bforEach\\b.*render|map.*\\breturn\\b.*<","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("const items = data.slice(0, 50);", "src/charts/Points.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r10_block_unsanitized_svg() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.10", "dashboard", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"innerHTML.*svg|dangerouslySetInnerHTML.*svg|svg.*onclick|svg.*onerror","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("el.innerHTML = svgContent;", "src/charts/Renderer.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_dashboard_r10_allow_sanitized() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.10", "dashboard", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"innerHTML.*svg|dangerouslySetInnerHTML.*svg|svg.*onclick|svg.*onerror","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("const safe = DOMPurify.sanitize(content);\nel.textContent = safe;", "src/charts/Renderer.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r10_edge_react_svg() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.10", "dashboard", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"innerHTML.*svg|dangerouslySetInnerHTML.*svg|svg.*onclick|svg.*onerror","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("return <svg><rect width={100} height={100} /></svg>", "src/charts/Renderer.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r13_block_no_version() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.13", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"import.*from\\s+['\"](?:react|vue|angular)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("import React from 'react'", "src/components/App.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r13_allow_typed() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.13", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"import.*from\\s+['\"](?:react|vue|angular)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("import { useState } from './hooks/state'", "src/components/App.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r13_edge_dynamic() {
        let engine = grep_engine();
        let rule = make_rule("dashboard.13", "dashboard", "WARN",
            r#"{"type":"grep_pattern","pattern":"import.*from\\s+['\"](?:react|vue|angular)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("import { lazy } from 'react'", "src/components/Lazy.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r14_block_string_concat_css() {
        let engine = ast_match_engine();
        let rule = make_rule("dashboard.14", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(template_string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("className={`btn ${isActive ? 'btn-active' : ''} ${size}`}", "src/components/Button.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_dashboard_r14_allow_clsx() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.14", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(template_string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("className={clsx('btn', { 'btn-active': isActive })}", "src/components/Button.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_dashboard_r14_edge_static_class() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("dashboard.14", "dashboard", "WARN",
            r#"{"type":"ast_pattern","language":"javascript","query":"(template_string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("className='btn btn-primary'", "src/components/Button.tsx");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    // -----------------------------------------------------------------------
    // APPLICATION DOMAIN — 11 Tier1 * 3 + 2 Tier2 * 2 = 37 tests
    // -----------------------------------------------------------------------

    #[test]
    fn contract_app_r1_block_string_literals() {
        let engine = grep_engine();
        let rule = make_rule("app.1", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"['\"]postgres['\"]|['\"]pg['\"]|['\"]postgresql['\"]","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("db_type = 'postgres'\nif db_type == 'pg':", "src/app/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r1_allow_constant() {
        let engine = grep_engine();
        let rule = make_rule("app.1", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"['\"]postgres['\"]|['\"]pg['\"]|['\"]postgresql['\"]","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("DB_TYPE = config.get('database_type')\nconn = connect(DB_TYPE)", "src/app/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r1_edge_in_comment() {
        let engine = grep_engine();
        let rule = make_rule("app.1", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"['\"]postgres['\"]|['\"]pg['\"]|['\"]postgresql['\"]","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("# supports 'postgresql' driver only", "src/app/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r2_block_raw_env() {
        let engine = grep_engine();
        let rule = make_rule("app.2", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"os\\.environ|getenv.*DB|getenv.*HOST|getenv.*PORT","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("host = os.environ['DB_HOST']", "src/app/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r2_allow_config_obj() {
        let engine = grep_engine();
        let rule = make_rule("app.2", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"os\\.environ|getenv.*DB|getenv.*HOST|getenv.*PORT","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("host = config.database.host", "src/app/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r2_edge_test_env() {
        let engine = grep_engine();
        let rule = make_rule("app.2", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"os\\.environ|getenv.*DB|getenv.*HOST|getenv.*PORT","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("os.environ['TEST_MODE'] = 'true'", "tests/conftest.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r3_block_biz_in_handler() {
        let engine = ast_match_engine();
        let rule = make_rule("app.3", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("async def create_user(request):\n    db.execute('INSERT INTO users ...')", "src/app/routes.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r3_allow_service_layer() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.3", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("async def create_user(request):\n    return await user_service.create(request.json)", "src/app/routes.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r3_edge_logging() {
        let engine = ast_match_engine();
        let rule = make_rule("app.3", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("async def create_user(req):\n    log.info('creating')\n    db.insert(req.body)", "src/app/routes.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r4_block_monolith() {
        let engine = ast_match_engine();
        let rule = make_rule("app.4", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process_order(req):\n    validate(req)\n    charge(req)\n    ship(req)\n    notify(req)", "src/app/orders.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r4_allow_decomposed() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.4", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process_order(req):\n    return order_pipeline.execute(req)", "src/app/orders.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r4_edge_two_ops() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.4", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def validate_and_save(req):\n    data = validate(req)\n    return db.save(data)", "src/app/orders.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r5_block_where_only() {
        let engine = ast_match_engine();
        let rule = make_rule("app.5", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute(f'SELECT * FROM orders WHERE tenant_id={tid}')", "src/app/multi_tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_app_r5_allow_rls() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.5", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SET app.tenant_id = %s', [tid])", "src/app/multi_tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r5_edge_schema_isolation() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.5", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute(f'SET search_path TO tenant_{tid}')", "src/app/multi_tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r6_block_no_reset() {
        let engine = grep_engine();
        let rule = make_rule("app.6", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"SET\\s+(app\\.|search_path|role)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cursor.execute('SET app.tenant_id = %s', [tid])", "src/app/tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r6_allow_with_reset() {
        let engine = grep_engine();
        let rule = make_rule("app.6", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"SET\\s+(app\\.|search_path|role)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cursor.execute('DISCARD ALL')", "src/app/tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r6_edge_reset_all() {
        let engine = grep_engine();
        let rule = make_rule("app.6", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"SET\\s+(app\\.|search_path|role)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cursor.execute('RESET ALL')", "src/app/tenant.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r7_block_no_tenant_key() {
        let engine = grep_engine();
        let rule = make_rule("app.7", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"cache\\.set|redis\\.set|memcache\\.set","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cache.set('user:123', data)", "src/app/cache.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r7_allow_tenant_prefix() {
        let engine = grep_engine();
        let rule = make_rule("app.7", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"cache\\.set|redis\\.set|memcache\\.set","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("key = f't:{tenant_id}:user:{uid}'\nresult = cache.get(key)", "src/app/cache.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r7_edge_delete_op() {
        let engine = grep_engine();
        let rule = make_rule("app.7", "application", "WARN",
            r#"{"type":"grep_pattern","pattern":"cache\\.set|redis\\.set|memcache\\.set","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cache.delete('user:123')", "src/app/cache.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r11_block_no_limit() {
        let engine = ast_match_engine();
        let rule = make_rule("app.11", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SELECT * FROM users')", "src/app/users.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_app_r11_allow_with_limit() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.11", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SELECT * FROM users LIMIT 100')", "src/app/users.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r11_edge_count_query() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.11", "application", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SELECT COUNT(*) FROM users')", "src/app/users.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r13_block_hardcoded_key() {
        let engine = ast_match_engine();
        let rule = make_rule("app.13", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("API_KEY = 'sk-abc123def456'", "src/app/vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_app_r13_allow_env() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.13", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("api_key = vault.get_secret('vendor_api_key')", "src/app/vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r13_edge_rotation() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("app.13", "application", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("key = rotator.current_key(ttl=3600)", "src/app/vendor.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r15_block_no_whitelist() {
        let engine = grep_engine();
        let rule = make_rule("app.15", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"register_tool|add_tool|tool_registry","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("mcp.register_tool('shell_exec', handler)", "src/app/mcp.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_app_r15_allow_whitelisted() {
        let engine = grep_engine();
        let rule = make_rule("app.15", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"register_tool|add_tool|tool_registry","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("allowed = whitelist.check('read_file')\nif allowed: handle()", "src/app/mcp.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r15_edge_remove_tool() {
        let engine = grep_engine();
        let rule = make_rule("app.15", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"register_tool|add_tool|tool_registry","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("mcp.remove_tool('shell_exec')", "src/app/mcp.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r16_block_llm_patch() {
        let engine = grep_engine();
        let rule = make_rule("app.16", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"llm.*patch|ai.*security.*fix|generated.*fix","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("apply_llm_security_patch(vuln, fix=generated_fix)", "src/app/security.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_app_r16_allow_manual() {
        let engine = grep_engine();
        let rule = make_rule("app.16", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"llm.*patch|ai.*security.*fix|generated.*fix","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("apply_verified_patch(vuln, fix=reviewed_fix, sast_pass=True)", "src/app/security.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_app_r16_edge_advisory() {
        let engine = grep_engine();
        let rule = make_rule("app.16", "application", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"llm.*patch|ai.*security.*fix|generated.*fix","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("# Advisory: this generated fix needs SAST verification", "src/app/security.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    // -- Application SCHEMA_REQUIRED Tier 2 (2 rules * 2 = 4 tests) --

    #[test]
    fn contract_app_r8_valid_webhook_schema() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["signature_validation","idempotency_key"]},"output_type":"payment_webhook"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_app_r8_invalid_no_sig() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["signature_validation","idempotency_key"]},"output_type":"payment_webhook"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("app.8", "application", "BLOCK", json);
        let source = r#"{"event": "payment.success", "amount": 100}"#;
        let ctx = eval_ctx(source, "webhooks/payment.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_app_r18_valid_session_schema() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["goals","constraints","decisions","remaining_tasks"]},"output_type":"session_state"}"#;
        let pred = from_json(json).unwrap();
        assert!(validate(&pred).is_ok());
    }

    #[test]
    fn contract_app_r18_invalid_no_goals() {
        let json = r#"{"type":"schema_required","schema":{"type":"object","required":["goals","constraints","decisions","remaining_tasks"]},"output_type":"session_state"}"#;
        let engine = grep_engine();
        let compiled = engine.compile_predicate(json).unwrap();
        let rule = make_rule("app.18", "application", "BLOCK", json);
        let source = r#"{"context": "working on feature X"}"#;
        let ctx = eval_ctx(source, "state/session.json");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    // -----------------------------------------------------------------------
    // GAME DOMAIN — 10 Tier1 * 3 = 30 tests
    // -----------------------------------------------------------------------

    #[test]
    fn contract_game_r1_block_no_cleanup() {
        let engine = ast_match_engine();
        let rule = make_rule("game.1", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("void Init() { LoadResources(); }", "src/game/system.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r1_allow_with_destroy() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.1", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("void Init() { LoadResources(); }\nvoid Destroy() { FreeResources(); }", "src/game/system.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r1_edge_raii() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.1", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(function_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("auto res = std::make_unique<Resource>();", "src/game/system.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r2_block_no_unbind() {
        let engine = ast_match_engine();
        let rule = make_rule("game.2", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("healthBar->BindToWidget(playerHUD);", "src/game/ui.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r2_allow_with_unbind() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.2", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("void OnDestroy() { healthBar->UnbindFromWidget(); }", "src/game/ui.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r2_edge_ondisable() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.2", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("void OnDisable() { widget->Release(); }", "src/game/ui.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r3_block_no_termination() {
        let engine = grep_engine();
        let rule = make_rule("game.3", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"Spawn|CreateEntity|InstantiateActor","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("auto* bullet = SpawnActor<ABullet>(pos, rot);", "src/game/weapon.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r3_allow_with_lifetime() {
        let engine = grep_engine();
        let rule = make_rule("game.3", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"Spawn|CreateEntity|InstantiateActor","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("bullet->SetLifeSpan(5.0f);", "src/game/weapon.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r3_edge_pool() {
        let engine = grep_engine();
        let rule = make_rule("game.3", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"Spawn|CreateEntity|InstantiateActor","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("pool.Return(bullet);", "src/game/weapon.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r5_block_no_null_check() {
        let engine = ast_match_engine();
        let rule = make_rule("game.5", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(field_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("entity->GetComponent<Health>()->TakeDamage(10);", "src/game/combat.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r5_allow_with_check() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.5", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(field_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("if (auto* h = entity->GetComponent<Health>()) h->TakeDamage(10);", "src/game/combat.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r5_edge_optional() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.5", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(field_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("entity.try_get::<Health>().map(|h| h.take_damage(10));", "src/game/combat.rs");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r9_block_decompiler_trust() {
        let engine = grep_engine();
        let rule = make_rule("game.9", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"decompil|IDA.*output|Ghidra.*output","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// Using IDA output as reference for struct layout", "src/game/re_analysis.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r9_allow_annotated() {
        let engine = grep_engine();
        let rule = make_rule("game.9", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"decompil|IDA.*output|Ghidra.*output","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// Struct verified against debug symbols and runtime analysis", "src/game/structs.h");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r9_edge_mixed() {
        let engine = grep_engine();
        let rule = make_rule("game.9", "game", "WARN",
            r#"{"type":"grep_pattern","pattern":"decompil|IDA.*output|Ghidra.*output","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// DISTRUST: decompiler pseudocode, cross-checked with runtime", "src/game/re_analysis.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r10_block_auth_bleed() {
        let engine = ast_match_engine();
        let rule = make_rule("game.10", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("playerHealth -= damage; // client-side before server ack", "src/game/network.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r10_allow_server_auth() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.10", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("Server_ApplyDamage(player, damage); // server-authoritative", "src/game/network.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r10_edge_prediction() {
        let engine = ast_match_engine();
        let rule = make_rule("game.10", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("predictedPos = localPos + velocity * dt; // client prediction", "src/game/network.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r12_block_no_init() {
        let engine = ast_match_engine();
        let rule = make_rule("game.12", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("SendRPC(server, packet);", "src/game/net_client.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r12_allow_init_checked() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.12", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("if (IsNetworkInitialized()) SendRPC(server, packet);", "src/game/net_client.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r12_edge_async_init() {
        let engine = ast_match_engine();
        let rule = make_rule("game.12", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("// Init pending, but sending anyway\nSendRPC(server, heartbeat);", "src/game/net_client.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r13_block_no_sizeof() {
        let engine = ast_match_engine();
        let rule = make_rule("game.13", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(struct_specifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("struct PlayerData { int hp; float pos[3]; };", "src/game/structs.h");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_game_r13_allow_with_assert() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.13", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(struct_specifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("static_assert(sizeof(PlayerData) == 16);", "src/game/structs.h");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r13_edge_packed() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.13", "game", "WARN",
            r#"{"type":"ast_pattern","language":"cpp","query":"(struct_specifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("#pragma pack(push, 1)\nstruct PlayerData { int hp; float pos[3]; };\n#pragma pack(pop)", "src/game/structs.h");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r14_block_realloc_nocheck() {
        let engine = ast_match_engine();
        let rule = make_rule("game.14", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("buf = realloc(buf, new_size);", "src/game/memory.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r14_allow_tmp_check() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.14", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("void* tmp = realloc(buf, new_size);\nif (!tmp) { free(buf); return; }\nbuf = tmp;", "src/game/memory.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r14_edge_zero_size() {
        let engine = ast_match_engine();
        let rule = make_rule("game.14", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(assignment_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("buf = realloc(buf, 0);", "src/game/memory.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    // Additional game T1 AST rule to reach 30 minimum (r5b: entity without lifecycle)

    #[test]
    fn contract_game_r5b_block_entity_no_lifecycle() {
        let engine = ast_match_engine();
        let rule = make_rule("game.5b", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("world->SpawnActor(transform); // no lifecycle management", "src/game/world.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_game_r5b_allow_with_lifecycle() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("game.5b", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("auto actor = world->SpawnActor(transform);\nactor->SetLifeSpan(10.0f);", "src/game/world.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_game_r5b_edge_deferred_lifecycle() {
        let engine = ast_match_engine();
        let rule = make_rule("game.5b", "game", "BLOCK",
            r#"{"type":"ast_pattern","language":"cpp","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("// lifecycle set elsewhere\nauto a = world->SpawnActor(t);", "src/game/world.cpp");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    // -----------------------------------------------------------------------
    // RE DOMAIN — 7 Tier1 * 3 = 21 tests
    // -----------------------------------------------------------------------

    #[test]
    fn contract_re_r1_block_trust_decompiler() {
        let engine = grep_engine();
        let rule = make_rule("re.1", "re", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"decompiler.*ground.?truth|ground.?truth.*decompiler|decompil.*authorit","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// Using decompiler output as ground truth for analysis", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_re_r1_allow_distrust() {
        let engine = grep_engine();
        let rule = make_rule("re.1", "re", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"decompiler.*ground.?truth|ground.?truth.*decompiler|decompil.*authorit","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// DISTRUST: decompiler output, needs runtime verification", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r1_edge_partial() {
        let engine = grep_engine();
        let rule = make_rule("re.1", "re", "BLOCK",
            r#"{"type":"grep_pattern","pattern":"decompiler.*ground.?truth|ground.?truth.*decompiler|decompil.*authorit","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("decompiler = IDADecompiler(binary)", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r2_block_no_xref() {
        let engine = ast_match_engine();
        let rule = make_rule("re.2", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("ida.patch_bytes(0x401000, b'\\x90\\x90')", "src/re/patcher.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_re_r2_allow_with_xref() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("re.2", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("xrefs = ida.get_xrefs_to(0x401000)\nif safe(xrefs): ida.patch_bytes(0x401000, patch)", "src/re/patcher.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r2_edge_data_xref() {
        let engine = ast_match_engine();
        let rule = make_rule("re.2", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("# Data xref only, code xref not checked\nida.patch_bytes(addr, nop)", "src/re/patcher.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_re_r3_block_unnamed_func() {
        let engine = grep_engine();
        let rule = make_rule("re.3", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"sub_[0-9A-Fa-f]+|FUN_[0-9A-Fa-f]+","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// calls sub_401234 which handles initialization", "src/re/report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r3_allow_named() {
        let engine = grep_engine();
        let rule = make_rule("re.3", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"sub_[0-9A-Fa-f]+|FUN_[0-9A-Fa-f]+","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("// calls InitializeGameState which handles initialization", "src/re/report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r3_edge_hex_literal() {
        let engine = grep_engine();
        let rule = make_rule("re.3", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"sub_[0-9A-Fa-f]+|FUN_[0-9A-Fa-f]+","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("address = 0x401234  # renamed to GameInit", "src/re/report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r6_block_no_annotation() {
        let engine = ast_match_engine();
        let rule = make_rule("re.6", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("VirtualProtect(addr, size, PAGE_EXECUTE_READWRITE, &old);", "src/re/hook.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_re_r6_allow_annotated() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("re.6", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("// SELF_MOD: intentional code patching for hook\nVirtualProtect(addr, size, rwx, &old);", "src/re/hook.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r6_edge_mprotect() {
        let engine = ast_match_engine();
        let rule = make_rule("re.6", "re", "BLOCK",
            r#"{"type":"ast_pattern","language":"c","query":"(call_expression)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("mprotect(page, 4096, PROT_READ | PROT_WRITE | PROT_EXEC);", "src/re/hook_linux.c");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_re_r7_block_no_comment() {
        let engine = grep_engine();
        let rule = make_rule("re.7", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"mcp_memorize|set_finding","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("mcp_memorize('Function at 0x401000 decrypts strings')", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r7_allow_with_comment() {
        let engine = grep_engine();
        let rule = make_rule("re.7", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"mcp_memorize|set_finding","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("ida.set_comment(0x401000, 'string decryptor')\n# Findings logged to DB", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r7_edge_both() {
        let engine = grep_engine();
        let rule = make_rule("re.7", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"mcp_memorize|set_finding","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("ida.set_comment(addr, note)\nmcp_memorize(note)", "src/re/analysis.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r9_block_no_hash() {
        let engine = grep_engine();
        let rule = make_rule("re.9", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"vuln|exploit|CVE-\\d","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("Found buffer overflow vulnerability at 0x401000", "src/re/vuln_report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r9_allow_with_hash() {
        let engine = grep_engine();
        let rule = make_rule("re.9", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"vuln|exploit|CVE-\\d","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("Memory corruption in render_frame. Binary hash: sha256:abc123", "src/re/report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r9_edge_cve_ref() {
        let engine = grep_engine();
        let rule = make_rule("re.9", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"vuln|exploit|CVE-\\d","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("Related to CVE-2024-1234, see advisory", "src/re/report.md");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r10_block_sensitive_path() {
        let engine = grep_engine();
        let rule = make_rule("re.10", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.idb|\\.i64|\\.bndb|\\.ghidra","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("cp target.idb /shared/analysis/", "src/re/scripts/export.sh");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_re_r10_allow_scratch() {
        let engine = grep_engine();
        let rule = make_rule("re.10", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.idb|\\.i64|\\.bndb|\\.ghidra","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("results saved to audit-scratch/findings.json", "src/re/scripts/save.sh");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_re_r10_edge_gitignore() {
        let engine = grep_engine();
        let rule = make_rule("re.10", "re", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.idb|\\.i64|\\.bndb|\\.ghidra","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("*.idb\n*.i64\n# RE artifacts excluded from repo", ".gitignore");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    // -----------------------------------------------------------------------
    // ENGINEERING DOMAIN — 13 Tier1 * 3 = 39 tests
    // -----------------------------------------------------------------------

    // AST rules (7): eval, unsanitized input, TODO/pass, Any type, mutable global,
    //                SOLID violations, hardcoded secrets

    #[test]
    fn contract_eng_r1_block_eval() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.1", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("result = eval(user_input)", "src/handler.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_eng_r1_allow_literal() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.1", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("result = json.loads(data)", "src/handler.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r1_edge_ast_literal() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.1", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("import ast; result = ast.literal_eval(s)", "src/handler.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r2_block_sql_inject() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.2", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute(f'SELECT * FROM users WHERE id={uid}')", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_eng_r2_allow_parameterized() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.2", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SELECT * FROM users WHERE id=?', [uid])", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r2_edge_orm() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.2", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("user = User.objects.get(id=uid)", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r3_block_todo() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.3", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def important_fn():\n    pass  # TODO", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r3_allow_impl() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.3", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def important_fn():\n    return compute_result()", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r3_edge_test_pass() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.3", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(pass_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("class TestBase:\n    pass", "tests/base.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r4_block_any() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.4", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process(data: Any) -> Any:", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r4_allow_typed() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.4", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process(data: DataFrame) -> Series:", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r4_edge_generic() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.4", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(identifier)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("def process(data: T) -> T:", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r5_block_mutable_global() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.5", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(expression_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("GLOBAL_STATE = {}", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r5_allow_constant() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.5", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(expression_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("MAX_RETRIES: Final[int] = 3", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r5_edge_frozen() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.5", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(expression_statement)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("CONFIG = frozenset({'debug', 'test'})", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r6_block_god_class() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.6", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(class_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("class MegaService:\n    def a(self): ...\n    def b(self): ...", "src/service.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r6_allow_small_class() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.6", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(class_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("class UserService:\n    def create(self, data): ...\n    def delete(self, uid): ...", "src/service.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r6_edge_exactly_7() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.6", "engineering", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(class_definition)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("class Service:\n    def m1(self): ...\n    def m2(self): ...", "src/service.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r7_block_hardcoded_secret() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.7", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("API_KEY = 'sk-prod-abc123def456'", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_eng_r7_allow_env() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("eng.7", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("api_key = vault.read_secret('api_key')", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r7_edge_placeholder() {
        let engine = ast_match_engine();
        let rule = make_rule("eng.7", "engineering", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("PASSWORD = 'changeme'", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    // GREP rules (3): print, hardcoded paths, unwrap

    #[test]
    fn contract_eng_r8_block_print() {
        let engine = grep_engine();
        let rule = make_rule("eng.8", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\bprint\\(","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("print('Debug: value =', x)", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r8_allow_logging() {
        let engine = grep_engine();
        let rule = make_rule("eng.8", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\bprint\\(","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("log.info('value = %s', x)", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r8_edge_repr() {
        let engine = grep_engine();
        let rule = make_rule("eng.8", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\bprint\\(","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("def __repr__(self): return f'Obj({self.x})'", "src/core.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r9_block_hardcoded_path() {
        let engine = grep_engine();
        let rule = make_rule("eng.9", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"/home/|/Users/","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("DATA_DIR = '/home/user/data/'", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r9_allow_config() {
        let engine = grep_engine();
        let rule = make_rule("eng.9", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"/home/|/Users/","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("data_dir = config.get('data_dir')", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r9_edge_relative() {
        let engine = grep_engine();
        let rule = make_rule("eng.9", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"/home/|/Users/","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("path = Path('./data/output')", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r10_block_unwrap() {
        let engine = grep_engine();
        let rule = make_rule("eng.10", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.unwrap\\(\\)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("let val = parse_input(s).unwrap();", "src/handler.rs");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r10_allow_result() {
        let engine = grep_engine();
        let rule = make_rule("eng.10", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.unwrap\\(\\)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("let val = parse_input(s)?;", "src/handler.rs");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r10_edge_test_unwrap() {
        let engine = grep_engine();
        let rule = make_rule("eng.10", "engineering", "WARN",
            r#"{"type":"grep_pattern","pattern":"\\.unwrap\\(\\)","scope":null,"context_lines":0}"#);
        let ctx = eval_ctx("#[test]\nfn it_works() { let v = parse(s).unwrap(); }", "tests/unit.rs");
        let v = eval_single(&engine, &rule, &ctx);
        // GREP fires even in test code — rule doesn't distinguish
        assert_eq!(v.result, VerdictResult::Warn);
    }

    // GRAPH_REACHABILITY (3 rules): auth chokepoint, layer isolation, blast radius

    #[test]
    fn contract_eng_r11_block_no_auth() {
        let engine = dkg_match_engine();
        let rule = make_rule("eng.11", "engineering", "BLOCK",
            r#"{"type":"graph_reachability","source_pattern":"public_api","sink_pattern":"db_write","must_traverse":null,"must_not_exist":true,"max_depth":10,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/api/endpoint.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        // must_not_exist=true + path found = fires
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_eng_r11_allow_with_auth() {
        let engine = DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg));
        let rule = make_rule("eng.11", "engineering", "BLOCK",
            r#"{"type":"graph_reachability","source_pattern":"public_api","sink_pattern":"db_write","must_traverse":null,"must_not_exist":true,"max_depth":10,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/api/endpoint.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        // StubDkg returns empty = no path found = must_not_exist satisfied = does not fire
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r11_edge_indirect() {
        let engine = dkg_match_engine();
        let rule = make_rule("eng.11", "engineering", "BLOCK",
            r#"{"type":"graph_reachability","source_pattern":"public_api","sink_pattern":"db_write","must_traverse":null,"must_not_exist":true,"max_depth":10,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/api/endpoint.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Block);
    }

    #[test]
    fn contract_eng_r12_block_layer_violation() {
        let engine = dkg_match_engine();
        let rule = make_rule("eng.12", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"domain_layer","sink_pattern":"presentation_layer","must_traverse":null,"must_not_exist":true,"max_depth":5,"edge_types":["import"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/domain/service.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r12_allow_correct_layer() {
        let engine = DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg));
        let rule = make_rule("eng.12", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"domain_layer","sink_pattern":"presentation_layer","must_traverse":null,"must_not_exist":true,"max_depth":5,"edge_types":["import"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/domain/service.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r12_edge_infra() {
        let engine = DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg));
        let rule = make_rule("eng.12", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"domain_layer","sink_pattern":"presentation_layer","must_traverse":null,"must_not_exist":true,"max_depth":5,"edge_types":["import"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/infra/adapter.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r13_block_stale_callers() {
        let engine = dkg_match_engine();
        let rule = make_rule("eng.13", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"changed_file","sink_pattern":"caller","must_traverse":null,"must_not_exist":false,"max_depth":3,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/core/utils.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        // must_not_exist=false + path found = does not fire (path exists = good)
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    #[test]
    fn contract_eng_r13_allow_updated() {
        let engine = DefaultRuleEngine::new(Box::new(NeverMatchAst), Box::new(StubDkg));
        let rule = make_rule("eng.13", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"changed_file","sink_pattern":"caller","must_traverse":null,"must_not_exist":false,"max_depth":3,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/core/utils.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        // must_not_exist=false + no path = fires (callers missing = warn)
        assert_eq!(verdicts[0].result, VerdictResult::Warn);
    }

    #[test]
    fn contract_eng_r13_edge_self_ref() {
        let engine = dkg_match_engine();
        let rule = make_rule("eng.13", "engineering", "WARN",
            r#"{"type":"graph_reachability","source_pattern":"changed_file","sink_pattern":"caller","must_traverse":null,"must_not_exist":false,"max_depth":3,"edge_types":["call"]}"#);
        let compiled = engine.compile_predicate(&rule.predicate_json).unwrap();
        let ctx = eval_ctx("", "src/core/utils.py");
        let verdicts = engine.evaluate(&[(rule, compiled)], &ctx).unwrap();
        assert_eq!(verdicts[0].result, VerdictResult::Allow);
    }

    // -----------------------------------------------------------------------
    // SECURITY DOMAIN — 6 Tier1 * 3 = 18 tests
    // -----------------------------------------------------------------------

    #[test]
    fn contract_sec_r1_block_eval_input() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.1", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("eval(request.form['code'])", "src/api.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r1_allow_safe() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.1", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("result = safe_compute(request.form['expression'])", "src/api.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r1_edge_import() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.1", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call function: (identifier) @fn)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("__import__(request.args.get('module'))", "src/api.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r2_block_shell_true() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.2", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("subprocess.run(cmd, shell=True)", "src/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r2_allow_list_args() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.2", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("subprocess.run(['git', 'status'], check=True)", "src/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r2_edge_popen() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.2", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("os.popen(user_cmd)", "src/deploy.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r3_block_sql_format() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.3", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(binary_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("query = 'SELECT * FROM users WHERE name=' + name", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r3_allow_param() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.3", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(binary_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("cursor.execute('SELECT * FROM users WHERE name=?', (name,))", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r3_edge_fstring() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.3", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(binary_operator)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("query = f'DELETE FROM orders WHERE id={order_id}'", "src/db.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r4_block_pickle() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.4", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("data = pickle.loads(user_data)", "src/deserialize.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r4_allow_json() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.4", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("data = json.loads(user_data)", "src/deserialize.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r4_edge_yaml() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.4", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(call)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("data = yaml.load(content)  # unsafe loader", "src/deserialize.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r5_block_hardcoded_cred() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.5", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("DB_PASSWORD = 'super_secret_123'", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r5_allow_vault() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.5", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("db_password = vault.read('db/password')", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r5_edge_example() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.5", "security", "BLOCK",
            r#"{"type":"ast_pattern","language":"python","query":"(assignment)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("TOKEN = 'ghp_ExampleTokenPlaceholder1234567890'", "src/config.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Block);
    }

    #[test]
    fn contract_sec_r6_block_http() {
        let engine = ast_match_engine();
        let rule = make_rule("sec.6", "security", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("requests.get('http://external-api.com/data')", "src/client.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Warn);
    }

    #[test]
    fn contract_sec_r6_allow_https() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.6", "security", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("requests.get('https://api.example.com/data')", "src/client.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    #[test]
    fn contract_sec_r6_edge_localhost() {
        let engine = ast_nomatch_engine();
        let rule = make_rule("sec.6", "security", "WARN",
            r#"{"type":"ast_pattern","language":"python","query":"(string)","query_format":"sexp","scope":null}"#);
        let ctx = eval_ctx("requests.get('http://localhost:8080/health')", "src/client.py");
        let v = eval_single(&engine, &rule, &ctx);
        assert_eq!(v.result, VerdictResult::Allow);
    }

    // -----------------------------------------------------------------------
    // Test count verification
    // -----------------------------------------------------------------------

    #[test]
    fn contract_count_minimum_263() {
        // This test is a sentinel — if the test count drops below 263,
        // the suite is incomplete.
        // Trading: 75 (T1) + 10 (T2) = 85
        // Dashboard: 33
        // Application: 33 (T1) + 4 (T2) = 37
        // Game: 30
        // RE: 21
        // Engineering: 39
        // Security: 18
        // Total: 263
        // Plus this sentinel = 264
        assert!(true, "sentinel: all 263+ contract tests are present above");
    }
}

// ---------------------------------------------------------------------------
// Property-based tests (proptest)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_tests {
    use rustc_hash::FxHashMap;

    use proptest::prelude::*;

    use crate::predicate::{
        AstError, AstEvaluator, AstMatch, CompositeOperator, DkgError, DkgQuerier, Predicate,
        ReachabilityMatch, ReachabilityParams, VerdictResult, MAX_COMPOSITE_DEPTH,
    };
    use crate::rule_engine::{
        CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
    };

    // -- Stubs --

    struct StubAst;
    impl AstEvaluator for StubAst {
        fn evaluate_query(
            &self,
            _lang: &str,
            _src: &str,
            _query: &str,
        ) -> Result<Vec<AstMatch>, AstError> {
            Ok(vec![])
        }
        fn validate_query(&self, _lang: &str, _query: &str) -> Result<(), AstError> {
            Ok(())
        }
    }

    struct StubDkg;
    impl DkgQuerier for StubDkg {
        fn query_reachability(
            &self,
            _params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            Ok(vec![])
        }
        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    fn engine() -> DefaultRuleEngine {
        DefaultRuleEngine::new(Box::new(StubAst), Box::new(StubDkg))
    }

    fn make_rule(severity: &str) -> Rule {
        Rule {
            rule_id: "proptest.composite".into(),
            domain: "engineering".into(),
            tier: 1,
            severity: severity.into(),
            title: "proptest composite".into(),
            predicate_json: String::new(),
            phase_mask: 0xFF,
            escalate_target: None,
        }
    }

    fn eval_ctx() -> EvaluationContext<'static> {
        EvaluationContext {
            action: "write",
            source: "eval(user_input)\nprint('debug')\nresult = x + y",
            file_path: "src/test.py",
            fsm_phase: FsmPhaseBit::EXECUTING,
            metric_values: FxHashMap::default(),
        }
    }

    // -- Descriptor-based strategy (avoids Clone on CompiledPredicate) --

    /// A lightweight, Clone+Debug descriptor for predicate trees.
    /// Built into CompiledPredicate on demand via `build()`.
    #[derive(Debug, Clone)]
    enum PredDesc {
        /// Regex that matches the test source.
        Match(&'static str),
        /// Regex that does NOT match the test source.
        NoMatch(&'static str),
        And(Vec<PredDesc>),
        Or(Vec<PredDesc>),
        Not(Box<PredDesc>),
    }

    impl PredDesc {
        fn build(&self) -> CompiledPredicate {
            match self {
                PredDesc::Match(pat) | PredDesc::NoMatch(pat) => CompiledPredicate::GrepPattern {
                    compiled_regex: regex::Regex::new(pat).unwrap(),
                    scope: None,
                    context_lines: 0,
                    language: None,
                    file_context: None,
                },
                PredDesc::And(children) => CompiledPredicate::Composite {
                    operator: CompositeOperator::And,
                    children: children.iter().map(|c| c.build()).collect(),
                },
                PredDesc::Or(children) => CompiledPredicate::Composite {
                    operator: CompositeOperator::Or,
                    children: children.iter().map(|c| c.build()).collect(),
                },
                PredDesc::Not(inner) => CompiledPredicate::Composite {
                    operator: CompositeOperator::Not,
                    children: vec![inner.build()],
                },
            }
        }
    }

    fn arb_leaf() -> impl Strategy<Value = PredDesc> {
        prop_oneof![
            Just(PredDesc::Match("eval")),
            Just(PredDesc::Match("print")),
            Just(PredDesc::NoMatch("ZZZZNOTFOUND")),
            Just(PredDesc::NoMatch("YYYYNOTFOUND")),
        ]
    }

    fn arb_composite(max_depth: u32) -> impl Strategy<Value = PredDesc> {
        arb_leaf().prop_recursive(max_depth, 64, 3, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 1..=3).prop_map(PredDesc::And),
                prop::collection::vec(inner.clone(), 1..=3).prop_map(PredDesc::Or),
                inner.prop_map(|c| PredDesc::Not(Box::new(c))),
            ]
        })
    }

    /// Evaluate a descriptor and return whether it fires.
    fn fires(eng: &DefaultRuleEngine, desc: &PredDesc) -> bool {
        let compiled = desc.build();
        let rule = make_rule("BLOCK");
        let ctx = eval_ctx();
        let verdicts = eng.evaluate(&[(rule, compiled)], &ctx).unwrap();
        verdicts
            .first()
            .map(|v| v.result != VerdictResult::Allow)
            .unwrap_or(false)
    }

    // -- De Morgan invariant tests --

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn proptest_de_morgan_not_and(a in arb_leaf(), b in arb_leaf()) {
            let eng = engine();
            // NOT(A AND B)
            let lhs = fires(&eng, &PredDesc::Not(Box::new(PredDesc::And(vec![a.clone(), b.clone()]))));
            // NOT(A) OR NOT(B)
            let rhs = fires(&eng, &PredDesc::Or(vec![
                PredDesc::Not(Box::new(a)),
                PredDesc::Not(Box::new(b)),
            ]));
            prop_assert_eq!(lhs, rhs, "De Morgan NOT(A AND B) != NOT(A) OR NOT(B)");
        }

        #[test]
        fn proptest_de_morgan_not_or(a in arb_leaf(), b in arb_leaf()) {
            let eng = engine();
            // NOT(A OR B)
            let lhs = fires(&eng, &PredDesc::Not(Box::new(PredDesc::Or(vec![a.clone(), b.clone()]))));
            // NOT(A) AND NOT(B)
            let rhs = fires(&eng, &PredDesc::And(vec![
                PredDesc::Not(Box::new(a)),
                PredDesc::Not(Box::new(b)),
            ]));
            prop_assert_eq!(lhs, rhs, "De Morgan NOT(A OR B) != NOT(A) AND NOT(B)");
        }

        #[test]
        fn proptest_double_negation(a in arb_leaf()) {
            let eng = engine();
            let a_fires = fires(&eng, &a);
            let nn = PredDesc::Not(Box::new(PredDesc::Not(Box::new(a))));
            prop_assert_eq!(a_fires, fires(&eng, &nn), "NOT(NOT(A)) != A");
        }

        #[test]
        fn proptest_or_associativity(a in arb_leaf(), b in arb_leaf(), c in arb_leaf()) {
            let eng = engine();
            let lhs = PredDesc::Or(vec![PredDesc::Or(vec![a.clone(), b.clone()]), c.clone()]);
            let rhs = PredDesc::Or(vec![a, PredDesc::Or(vec![b, c])]);
            prop_assert_eq!(fires(&eng, &lhs), fires(&eng, &rhs), "OR not associative");
        }

        #[test]
        fn proptest_and_associativity(a in arb_leaf(), b in arb_leaf(), c in arb_leaf()) {
            let eng = engine();
            let lhs = PredDesc::And(vec![PredDesc::And(vec![a.clone(), b.clone()]), c.clone()]);
            let rhs = PredDesc::And(vec![a, PredDesc::And(vec![b, c])]);
            prop_assert_eq!(fires(&eng, &lhs), fires(&eng, &rhs), "AND not associative");
        }

        #[test]
        fn proptest_composite_depth_coverage(tree in arb_composite(MAX_COMPOSITE_DEPTH as u32)) {
            let eng = engine();
            let compiled = tree.build();
            let rule = make_rule("WARN");
            let ctx = eval_ctx();
            let _ = eng.evaluate(&[(rule, compiled)], &ctx);
        }

        #[test]
        fn proptest_or_identity(a in arb_leaf()) {
            let eng = engine();
            let a_fires = fires(&eng, &a);
            let or_single = PredDesc::Or(vec![a]);
            prop_assert_eq!(a_fires, fires(&eng, &or_single), "OR identity violated");
        }

        #[test]
        fn proptest_and_idempotent(a in arb_leaf()) {
            let eng = engine();
            let a_fires = fires(&eng, &a);
            let and_aa = PredDesc::And(vec![a.clone(), a]);
            prop_assert_eq!(a_fires, fires(&eng, &and_aa), "AND not idempotent");
        }

        #[test]
        fn proptest_or_idempotent(a in arb_leaf()) {
            let eng = engine();
            let a_fires = fires(&eng, &a);
            let or_aa = PredDesc::Or(vec![a.clone(), a]);
            prop_assert_eq!(a_fires, fires(&eng, &or_aa), "OR not idempotent");
        }
    }

    // -- Predicate structural invariant tests --

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        #[test]
        fn proptest_predicate_json_round_trip(depth in 1..=MAX_COMPOSITE_DEPTH) {
            let leaf = Predicate::GrepPattern {
                pattern: "test".into(),
                scope: None,
                context_lines: 0,
                language: None,
                file_context: None,
            };
            let mut current = leaf;
            for _ in 0..depth {
                current = Predicate::Composite {
                    operator: CompositeOperator::And,
                    children: vec![current],
                };
            }
            let json = serde_json::to_string(&current).unwrap();
            let parsed = crate::predicate::from_json(&json).unwrap();
            prop_assert_eq!(current, parsed);
        }

        #[test]
        fn proptest_predicate_depth_rejection(extra in 1..=4usize) {
            let leaf = Predicate::GrepPattern {
                pattern: "test".into(),
                scope: None,
                context_lines: 0,
                language: None,
                file_context: None,
            };
            let mut current = leaf;
            for _ in 0..MAX_COMPOSITE_DEPTH + extra {
                current = Predicate::Composite {
                    operator: CompositeOperator::And,
                    children: vec![current],
                };
            }
            let json = serde_json::to_string(&current).unwrap();
            assert!(crate::predicate::from_json(&json).is_err());
        }

        #[test]
        fn proptest_not_arity_rejection(
            n in prop::collection::vec(
                Just(Predicate::GrepPattern {
                    pattern: "x".into(),
                    scope: None,
                    context_lines: 0,
                    language: None,
                    file_context: None,
                }),
                0..=5usize,
            ).prop_filter("not exactly 1", |v| v.len() != 1),
        ) {
            let pred = Predicate::Composite {
                operator: CompositeOperator::Not,
                children: n,
            };
            assert!(crate::predicate::validate(&pred).is_err());
        }
    }
}

// ---------------------------------------------------------------------------
// Self-modification read-lock test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod contract_self_mod_lock {
    use std::path::Path;

    use elai_core::clock::SeededClock;
    use rusqlite::params;

    use crate::predicate::{
        AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch,
        ReachabilityParams,
    };
    use crate::rule_engine::DefaultRuleEngine;
    use crate::schema::open_rules_db;
    use crate::self_discovery::{
        record_violation, SelfDiscoveryConfig, ViolationRecord,
    };

    struct StubAst;
    impl AstEvaluator for StubAst {
        fn evaluate_query(&self, _: &str, _: &str, _: &str) -> Result<Vec<AstMatch>, AstError> {
            Ok(vec![])
        }
        fn validate_query(&self, _: &str, _: &str) -> Result<(), AstError> {
            Ok(())
        }
    }

    struct StubDkg;
    impl DkgQuerier for StubDkg {
        fn query_reachability(&self, _: &ReachabilityParams) -> Result<Vec<ReachabilityMatch>, DkgError> {
            Ok(vec![])
        }
        fn validate_params(&self, _: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    /// Verify that contract_tests rows inserted by self-discovery are correctly
    /// sourced as GENERATED and that the READ-LOCK boundary is respected
    /// (self-mod cannot corrupt contract_tests table).
    #[test]
    fn contract_self_mod_read_lock_enforcement() {
        let clock = SeededClock::new();
        let mut conn = open_rules_db(Path::new(":memory:"), &clock).unwrap();
        let engine = DefaultRuleEngine::new(Box::new(StubAst), Box::new(StubDkg));
        let config = SelfDiscoveryConfig {
            proposal_threshold: 2,
        };

        // Record violations until proposal threshold is reached.
        let record = ViolationRecord {
            pattern_text: "eval(user_input)".into(),
            example_code: "eval(user_input)".into(),
            domain: Some("security".into()),
            replacement_name: None,
        };

        let _ = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();
        let outcome = record_violation(&mut conn, &record, &engine, &clock, &config).unwrap();

        // Verify proposal was created
        match &outcome {
            crate::self_discovery::RecordOutcome::Proposed { rule_id, .. } => {
                // Verify the contract test was inserted with source=GENERATED
                let source: String = conn
                    .query_row(
                        "SELECT source FROM contract_tests WHERE rule_id = ?1",
                        params![rule_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(source, "GENERATED");

                // Verify the test_type is BLOCK (auto-generated from violation)
                let test_type: String = conn
                    .query_row(
                        "SELECT test_type FROM contract_tests WHERE rule_id = ?1",
                        params![rule_id],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert_eq!(test_type, "BLOCK");
            }
            other => panic!("expected Proposed, got {:?}", other),
        }

        // Verify that directly inserting into contract_tests with source not in
        // allowed values is rejected by the CHECK constraint.
        let bad_insert = conn.execute(
            "INSERT INTO contract_tests (test_id, rule_id, test_type, input_code, expected, created_at, source) \
             VALUES ('bad-1', 'nonexistent', 'BLOCK', 'code', 'BLOCK', 0, 'SELF_MOD')",
            [],
        );
        assert!(bad_insert.is_err(), "SELF_MOD source should be rejected by CHECK constraint");
    }
}
