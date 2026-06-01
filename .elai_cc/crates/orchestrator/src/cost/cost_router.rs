//! E-39 -- pre-execution difficulty estimator + budget-aware route
//! decision (FrugalGPT-style cascade per arXiv:2305.05176).
//!
//! Ship-standalone classifier with a 10-task minimal reference corpus
//! (C-46 `TaskDifficultyClassifier` is PLANNED but not shipped; the
//! upgrade path swaps this module's [`TaskDifficultyClassifier`] for
//! C-46's once it lands).
//!
//! The router maps a `(difficulty, budget_remaining_usd, criticality)`
//! triple to one of:
//! * [`RouteDecision::Local`] -- run on the local sidecar
//! * [`RouteDecision::Sonnet`] -- cloud Sonnet 4.6 ($3/$15 per MTok)
//! * [`RouteDecision::Haiku`]  -- cloud Haiku 4.5 ($1/$5 per MTok)
//! * [`RouteDecision::DeepSeekFlash`] -- DeepSeek V4-Flash ($0.30/$0.50)
//! * [`RouteDecision::Opus { requires_confirmation }`] -- Opus 4.7
//!   ($5/$25, gated behind typed-confirmation per plan stub_ban
//!   `no all-Opus default`)
//!
//! Per arXiv:2604.22750 ("How Do AI Agents Spend Your Money?",
//! Stanford Digital Economy Lab, verified May 2026 via WebSearch at
//! arxiv.org/abs/2604.22750 + alphaxiv.org/overview/2604.22750):
//! same-task variance reaches 30x, frontier models systematically
//! underestimate their own cost (max correlation 0.39), accuracy
//! often peaks at intermediate cost. The router treats both
//! difficulty and remaining-budget as a P50+P95 band rather than a
//! point.

use rustc_hash::FxHashMap;

/// Difficulty score in `[0.0, 1.0]`. 0.0 = trivial, 1.0 = frontier-
/// hard. Combined with `budget_remaining_usd` to pick a backend.
pub type Difficulty = f32;

/// User-supplied criticality tier. Maps to the report's
/// $50/$100/$1000 layered-cap rocketedge.com 2026 schema: low =
/// scaffold work, medium = standard ELAI baby-step, high = critical
/// path step on a payment / safety / compliance flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Criticality {
    Low,
    Medium,
    High,
}

impl Criticality {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// 5-arm route decision. Opus carries a `requires_confirmation` flag
/// per plan stub_ban: "no all-Opus default routing" + "very-hard +
/// critical with explicit user confirmation only".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteDecision {
    Local,
    Sonnet,
    Haiku,
    DeepSeekFlash,
    Opus { requires_confirmation: bool },
}

impl RouteDecision {
    /// Snake-case label for SSB / dashboard / TS-mirror surfaces.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Sonnet => "sonnet",
            Self::Haiku => "haiku",
            Self::DeepSeekFlash => "deepseek_v4_flash",
            Self::Opus { .. } => "opus",
        }
    }
}

/// Difficulty bins. Boundaries calibrated on the 10-task reference
/// corpus per E-39-decisions.md DECISION-4. Tunable via
/// `[cloud.cost_router]` once empirical data lands.
const DIFFICULTY_EASY_CEIL: Difficulty = 0.33;
const DIFFICULTY_HARD_CEIL: Difficulty = 0.66;

/// Budget thresholds (USD) for the tight/ample split. Aligned with the
/// rocketedge.com 2026 layered-cap schema ($50 soft / $100 hard /
/// $1000 monthly manual-override).
const BUDGET_TIGHT_CEIL_USD: f64 = 5.0;
const BUDGET_AMPLE_FLOOR_USD: f64 = 25.0;

/// Decision matrix per plan Task 5. Pulled into a free function so a
/// unit test enumerates the full grid.
#[must_use]
pub fn route_for_difficulty_and_budget(
    diff: Difficulty,
    budget_remaining_usd: f64,
    criticality: Criticality,
) -> RouteDecision {
    let easy = diff <= DIFFICULTY_EASY_CEIL;
    let hard = diff > DIFFICULTY_EASY_CEIL && diff <= DIFFICULTY_HARD_CEIL;
    let very_hard = diff > DIFFICULTY_HARD_CEIL;
    let tight = budget_remaining_usd <= BUDGET_TIGHT_CEIL_USD;
    let ample = budget_remaining_usd >= BUDGET_AMPLE_FLOOR_USD;

    // Plan Task 5 matrix:
    //   (easy + ample)        -> Local       (cheap; local handles it)
    //   (easy + tight)        -> Local       (cheap; budget-defensive)
    //   (hard + ample)        -> Sonnet      (workhorse)
    //   (hard + tight)        -> DeepSeekFlash OR Haiku (cheaper cloud)
    //   (very_hard + critical High + ample/medium) -> Opus { needs confirm }
    //   (very_hard + tight)   -> DeepSeekFlash (last-resort cheap path)
    //   (very_hard + Low)     -> Sonnet      (don't burn Opus on low criticality)
    if easy {
        return RouteDecision::Local;
    }
    if hard {
        if ample {
            return RouteDecision::Sonnet;
        }
        if tight {
            return RouteDecision::DeepSeekFlash;
        }
        return RouteDecision::Haiku;
    }
    if very_hard {
        match criticality {
            Criticality::High => RouteDecision::Opus {
                requires_confirmation: true,
            },
            Criticality::Medium => {
                if tight {
                    RouteDecision::DeepSeekFlash
                } else {
                    RouteDecision::Sonnet
                }
            }
            Criticality::Low => {
                if tight {
                    RouteDecision::DeepSeekFlash
                } else {
                    RouteDecision::Sonnet
                }
            }
        }
    } else {
        // Shouldn't be reachable given the easy/hard/very_hard split,
        // but the explicit fallback keeps the match exhaustive on
        // future bin tweaks without panicking.
        RouteDecision::Sonnet
    }
}

/// Heuristic difficulty classifier. Ships with a 10-task minimal
/// corpus (DECISION-4); upgrade path swaps to C-46's classifier once
/// shipped. Features:
/// * `trigram_unique_ratio` -- unique-trigram count / total-trigram
///   count, capped at 1.0; higher = more lexical variety = harder.
/// * `length_factor` -- log10(1 + chars) / 4, capped at 1.0; longer
///   tasks tend to be harder.
/// * `keyword_signal` -- count of "frontier" keywords (algorithm,
///   prove, optimize, theorem, derive, parallel, lock-free, monad,
///   formal-verify) divided by 8 (cap).
///
/// The 3 features are averaged. The classifier is intentionally
/// simple; correctness comes from the gate-the-cloud-call discipline
/// (every dispatch logs a [`crate::ssb::CostEventKind::CostRouterDecision`]
/// row), not from any single classifier's precision.
pub struct TaskDifficultyClassifier {
    keyword_set: FxHashMap<&'static str, u8>,
}

impl Default for TaskDifficultyClassifier {
    fn default() -> Self {
        let mut keyword_set = FxHashMap::default();
        for k in [
            "algorithm",
            "prove",
            "optimize",
            "theorem",
            "derive",
            "parallel",
            "lock-free",
            "monad",
            "formal-verify",
            "concurrency",
        ] {
            keyword_set.insert(k, 1);
        }
        Self { keyword_set }
    }
}

impl TaskDifficultyClassifier {
    /// Construct with the default 10-keyword corpus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn trigram_unique_ratio(text: &str) -> f32 {
        let bytes: Vec<char> = text.chars().collect();
        if bytes.len() < 3 {
            return 0.0;
        }
        let mut seen = FxHashMap::<(char, char, char), u32>::default();
        let mut total: u32 = 0;
        for w in bytes.windows(3) {
            total += 1;
            let trigram = (w[0], w[1], w[2]);
            *seen.entry(trigram).or_insert(0) += 1;
        }
        if total == 0 {
            return 0.0;
        }
        (seen.len() as f32 / total as f32).min(1.0)
    }

    fn length_factor(text: &str) -> f32 {
        let l = text.chars().count() as f32;
        let f = ((1.0 + l).log10()) / 4.0;
        f.clamp(0.0, 1.0)
    }

    fn keyword_signal(&self, text: &str) -> f32 {
        let lower = text.to_ascii_lowercase();
        let mut hits: u32 = 0;
        for k in self.keyword_set.keys() {
            if lower.contains(k) {
                hits += 1;
            }
        }
        (hits as f32 / 8.0).clamp(0.0, 1.0)
    }

    /// Compute a difficulty score for the given task description.
    /// Returns a value in `[0.0, 1.0]`.
    #[must_use]
    pub fn classify(&self, task_description: &str) -> Difficulty {
        let a = Self::trigram_unique_ratio(task_description);
        let b = Self::length_factor(task_description);
        let c = self.keyword_signal(task_description);
        ((a + b + c) / 3.0).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easy_ample_routes_local() {
        let d = route_for_difficulty_and_budget(0.10, 50.0, Criticality::Low);
        assert_eq!(d, RouteDecision::Local);
    }

    #[test]
    fn easy_tight_routes_local() {
        let d = route_for_difficulty_and_budget(0.10, 1.0, Criticality::Low);
        assert_eq!(d, RouteDecision::Local);
    }

    #[test]
    fn hard_ample_routes_sonnet() {
        let d = route_for_difficulty_and_budget(0.50, 50.0, Criticality::Medium);
        assert_eq!(d, RouteDecision::Sonnet);
    }

    #[test]
    fn hard_tight_routes_deepseek() {
        let d = route_for_difficulty_and_budget(0.50, 1.0, Criticality::Medium);
        assert_eq!(d, RouteDecision::DeepSeekFlash);
    }

    #[test]
    fn very_hard_critical_routes_opus_with_confirmation() {
        let d = route_for_difficulty_and_budget(0.90, 100.0, Criticality::High);
        match d {
            RouteDecision::Opus {
                requires_confirmation,
            } => assert!(requires_confirmation),
            other => panic!("expected Opus, got {other:?}"),
        }
    }

    #[test]
    fn very_hard_low_criticality_does_not_route_opus() {
        let d = route_for_difficulty_and_budget(0.90, 100.0, Criticality::Low);
        assert!(!matches!(d, RouteDecision::Opus { .. }));
    }

    #[test]
    fn no_all_opus_default_under_100_calls() {
        // Plan stub_ban: "no all-Opus default (anti-pattern); <10% Opus
        // on default config". 100 synthetic decisions with default
        // criticality=Medium produces 0 Opus rows.
        let mut opus_count = 0;
        let classifier = TaskDifficultyClassifier::new();
        let tasks = [
            "rename a variable",
            "add a unit test for foo",
            "wire a new tauri command",
            "fix a typo in a doc comment",
            "implement a simple parser",
            "add a config field",
            "refactor the spend accumulator",
            "extract a helper function",
            "fix a clippy warning",
            "update the changelog",
        ];
        for t in tasks.iter().cycle().take(100) {
            let diff = classifier.classify(t);
            let d = route_for_difficulty_and_budget(diff, 30.0, Criticality::Medium);
            if matches!(d, RouteDecision::Opus { .. }) {
                opus_count += 1;
            }
        }
        assert!(opus_count < 10, "expected <10% Opus, got {opus_count}");
    }

    #[test]
    fn classifier_returns_bounded_score() {
        let c = TaskDifficultyClassifier::new();
        let d = c.classify("hello world");
        assert!((0.0..=1.0).contains(&d));
    }

    #[test]
    fn classifier_returns_higher_score_for_keyword_dense_task() {
        let c = TaskDifficultyClassifier::new();
        let easy = c.classify("rename a variable");
        let hard = c.classify(
            "prove the optimize algorithm using a formal-verify theorem for a lock-free monad in a parallel concurrency derive proof",
        );
        assert!(
            hard > easy,
            "keyword-dense task must score higher: easy={easy}, hard={hard}"
        );
    }

    #[test]
    fn route_decision_labels_stable() {
        assert_eq!(RouteDecision::Local.as_str(), "local");
        assert_eq!(RouteDecision::Sonnet.as_str(), "sonnet");
        assert_eq!(RouteDecision::Haiku.as_str(), "haiku");
        assert_eq!(RouteDecision::DeepSeekFlash.as_str(), "deepseek_v4_flash");
        assert_eq!(
            RouteDecision::Opus {
                requires_confirmation: true
            }
            .as_str(),
            "opus"
        );
    }
}
