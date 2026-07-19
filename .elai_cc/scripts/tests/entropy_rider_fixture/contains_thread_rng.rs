// smoke fixture: scanner MUST report this file (unannotated thread_rng)
fn fixture_rng() {
    let v: u32 = rand::thread_rng().gen();
    println!("{}", v);
}
