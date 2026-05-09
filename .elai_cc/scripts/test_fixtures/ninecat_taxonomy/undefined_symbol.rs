// Category: UndefinedSymbol
// Expected compiler error: E0425 cannot find function `foo_undefined`
// cargo check produces: "not found in this scope"
fn main() {
    foo_undefined();
}
