"""Mirror of crates/contracts/tests/common/mod.rs::is_sibling_negative_space_contract.

Validates the predicate matches `e<NUMBER>_*_contracts.rs` and only that.
This stdlib-only re-implementation should byte-for-byte mirror the Rust:

    fn is_sibling_negative_space_contract(name: &str) -> bool {
        let Some(rest) = name.strip_prefix('e') else { return false; };
        let mut chars = rest.chars();
        let first = chars.next();
        if !first.map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return false;
        }
        let after_digits: String = chars.by_ref().skip_while(|c| c.is_ascii_digit()).collect();
        after_digits.starts_with('_') && after_digits.ends_with("_contracts.rs")
    }
"""

import sys


def is_sibling_negative_space_contract(name: str) -> bool:
    if not name.startswith("e"):
        return False
    rest = name[1:]
    if not rest:
        return False
    first = rest[0]
    if not (first.isascii() and first.isdigit()):
        return False
    # skip remaining digits
    idx = 0
    while idx < len(rest) and rest[idx].isascii() and rest[idx].isdigit():
        idx += 1
    after_digits = rest[idx:]
    return after_digits.startswith("_") and after_digits.endswith("_contracts.rs")


CASES = [
    # (name, expected, reason)
    ("e11_idea_to_saas_contracts.rs", True, "canonical sibling"),
    ("e12_production_codegen_contracts.rs", True, "canonical sibling"),
    ("e25_compaction_architecture_contracts.rs", True, "canonical sibling"),
    ("e38_anti_pattern_contracts.rs", True, "canonical sibling"),
    ("e48_research_contracts.rs", True, "canonical sibling"),
    ("e0_contracts.rs", False, "missing _<tail>_ between digits and _contracts.rs (after_digits='_contracts.rs' starts with _ AND ends with _contracts.rs — TRUE per spec, BUT prompt says SHOULD match)"),
    ("e_contracts.rs", False, "missing digit"),
    ("extra_contracts.rs", False, "letters after 'e', no digit"),
    ("e1_x_contracts.rs", True, "single digit + tail"),
    ("e123_anything_contracts.rs", True, "multi-digit"),
    ("e11_idea_to_saas_contract.rs", False, "no plural 'contracts'"),
    ("e11_idea_to_saas_contracts.txt", False, "wrong ext"),
    ("Ee11_contracts.rs", False, "uppercase prefix"),
    ("a18_generation_profile_contract.rs", False, "different letter"),
    ("c11_sandbox_isolation_contract.rs", False, "different letter prefix"),
    ("e11_idea.rs", False, "no contracts suffix"),
    ("e11contracts.rs", False, "missing underscore between number and tail"),
    ("e11_contracts_x.rs", False, "trailing token after _contracts"),
    ("e2a_contracts.rs", False, "non-digit after digits prefix consumed = 'a_contracts.rs' starts with 'a' not '_'"),
]


def main() -> int:
    print(f"Mirror predicate validation — {len(CASES)} cases\n")
    failures = []
    for name, expected, reason in CASES:
        actual = is_sibling_negative_space_contract(name)
        status = "OK" if actual == expected else "FAIL"
        if actual != expected:
            failures.append((name, expected, actual, reason))
        print(f"  [{status}] {name!r:55s} expected={expected!s:5s} actual={actual!s:5s}  ({reason})")
    print()
    if failures:
        print(f"FAIL: {len(failures)} mismatches")
        for name, exp, act, reason in failures:
            print(f"   - {name!r}: expected {exp}, got {act} ({reason})")
        return 1
    print(f"PASS: all {len(CASES)} cases match")

    # Special prompt-specified edge cases reported separately:
    print()
    print("Prompt-specified edge cases:")
    e_only = is_sibling_negative_space_contract("e_contracts.rs")
    print(f"  e_contracts.rs => {e_only}  (prompt: should NOT match — got {e_only}, {'OK' if not e_only else 'BAD'})")
    e0 = is_sibling_negative_space_contract("e0_contracts.rs")
    print(f"  e0_contracts.rs => {e0}  (prompt: SHOULD match — predicate returns {e0})")
    extra = is_sibling_negative_space_contract("extra_contracts.rs")
    print(f"  extra_contracts.rs => {extra}  (prompt: should NOT match — got {extra}, {'OK' if not extra else 'BAD'})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
