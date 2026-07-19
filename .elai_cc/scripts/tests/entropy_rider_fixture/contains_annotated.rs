// smoke fixture: scanner MUST NOT report this (entropy-allow: reason in allowlist)
fn fixture_annotated() {
    let id = uuid::Uuid::new_v4(); // entropy-allow: fixture-annotated-smoke-allowlist-reason
    println!("{}", id);
}
