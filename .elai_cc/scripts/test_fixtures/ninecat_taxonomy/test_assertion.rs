// Category: TestAssertion
// Expected: assertion failure in test (not a compile error but a test failure).
// cargo test produces: "thread 'it_fails' panicked at 'assertion `left == right` failed'"
// Note: cargo check does NOT fail on this; cargo test does.
// The classifier sees the test output from `cargo test`, not `cargo check`.

#[test]
fn it_fails() {
    assert_eq!(1, 2, "left=1 right=2 mismatch");
}

fn main() {}
