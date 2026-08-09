//! N-1 measurement spine: the ratified registry seam for profile-producing benches.
//!
//! Only two bench handlers write a `ModelCapabilityProfile` measurement sink today,
//! so the registry has exactly two rows. Absent or planned benches get no row.

/// One closed measurement-spine registry row.
pub struct SpineEntry {
    pub bench_family: &'static str,
    pub cli_action: &'static str,
    pub profile_sinks: &'static [&'static str],
}

/// Canonical number of profile-producing bench families.
pub const EXPECTED_SPINE_COUNT: usize = 2;

static ENTRIES: [SpineEntry; EXPECTED_SPINE_COUNT] = [
    SpineEntry {
        bench_family: "context-fill",
        cli_action: "BenchContextFill",
        profile_sinks: &["fill_degradation_curve", "calibrated_trim_fraction"],
    },
    SpineEntry {
        bench_family: "positional-probe",
        cli_action: "BenchPositionalProbe",
        profile_sinks: &["positional_value_curve", "ordering_seed"],
    },
];

/// Return the canonical measurement-spine registry.
pub fn spine_entries() -> &'static [SpineEntry] {
    &ENTRIES
}

/// Probe-driven dark report. Returns indexed families whose CLI route is dark.
pub fn dark_families(route_live: impl Fn(&str) -> bool) -> Vec<&'static str> {
    spine_entries()
        .iter()
        .filter_map(|entry| (!route_live(entry.bench_family)).then_some(entry.bench_family))
        .collect()
}

/// Render the index table, marking each indexed family LIVE or DARK.
pub fn render_spine_table(dark: &[&str]) -> String {
    let mut output = String::from("measurement spine index\n");
    for entry in spine_entries() {
        let status = if dark.contains(&entry.bench_family) { "DARK" } else { "LIVE" };
        output.push_str(&format!(
            "{status} {} {} sinks: {}\n",
            entry.bench_family,
            entry.cli_action,
            entry.profile_sinks.join(", "),
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_canonical_size() {
        assert_eq!(spine_entries().len(), EXPECTED_SPINE_COUNT);
    }

    #[test]
    fn registry_has_no_ghost_rows() {
        for entry in spine_entries() {
            assert!(!["reasoning-knee", "timeliness", "commit-mined"].contains(&entry.bench_family));
        }
    }

    #[test]
    fn all_routes_are_live() {
        assert!(dark_families(|_| true).is_empty());
    }

    #[test]
    fn reports_dark_context_fill() {
        assert_eq!(dark_families(|family| family != "context-fill"), ["context-fill"]);
    }

    #[test]
    fn rendered_table_names_rows_and_sinks() {
        let table = render_spine_table(&[]);
        for text in [
            "context-fill", "BenchContextFill", "fill_degradation_curve",
            "calibrated_trim_fraction", "positional-probe", "BenchPositionalProbe",
            "positional_value_curve", "ordering_seed", "LIVE",
        ] {
            assert!(table.contains(text), "missing {text}");
        }
    }
}
