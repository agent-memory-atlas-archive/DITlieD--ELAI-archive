// Category: TypeMismatch
// Expected compiler error: E0308 mismatched types
// cargo check produces: "expected `i32`, found `&str`"
fn main() {
    let x: i32 = "hello";
    let _ = x;
}
