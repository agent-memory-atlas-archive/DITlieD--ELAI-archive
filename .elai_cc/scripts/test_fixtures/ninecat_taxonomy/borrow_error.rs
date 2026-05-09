// Category: BorrowError
// Expected compiler error: E0499 cannot borrow `v` as mutable more than once at a time
// cargo check produces: "second mutable borrow occurs here"
fn main() {
    let mut v = vec![1, 2, 3];
    let a = &mut v;
    let b = &mut v;
    let _ = (a, b);
}
