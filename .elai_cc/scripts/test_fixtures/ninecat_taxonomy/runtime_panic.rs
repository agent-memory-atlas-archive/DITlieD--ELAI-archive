// Category: RuntimePanic
// Expected: index out of bounds at runtime (cargo check/test passes compile;
// runtime produces "index out of bounds: the len is 0 but the index is 99").
// cargo test produces: "thread 'main' panicked at 'index out of bounds'"
// Note: this is a runtime error, not compile-time. The classifier sees the
// panic message from `cargo run` or test output.

fn main() {
    let v: Vec<i32> = vec![];
    // This compiles fine but panics at runtime
    println!("{}", v[99]);
}
