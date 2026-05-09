// Category: LifetimeError
// Expected compiler error: E0515 cannot return reference to local variable `s`
// cargo check produces: "returns a reference to data owned by the current function"
fn get_str<'a>() -> &'a str {
    let s = String::from("x");
    &s
}

fn main() {
    let _s = get_str();
}
