// (in tests/ dir -- path-excluded, scanner MUST NOT report)
fn fixture_in_tests() {
    let _ = rand::thread_rng().gen::<u32>();
}
