// Category: UnusedWarning
// deny(unused_variables) escalates warning to error.
// Expected compiler error: E0unused / "unused variable `_x`" as error
// cargo check produces: "error: unused variable: `x`"
#![deny(unused_variables)]

fn main() {
    let x = 1;
    println!("value: {}", x + 0);
    let unused = 42;
    // unused is never read — triggers deny(unused_variables) as error
    let _ = unused;
}
