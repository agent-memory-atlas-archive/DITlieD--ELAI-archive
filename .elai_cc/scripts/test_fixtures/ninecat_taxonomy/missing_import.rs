// Category: MissingImport
// Expected compiler error: E0412 cannot find type `HashSet` in this scope
// HashMap is imported but HashSet is used without import.
// cargo check produces: "consider importing: `use std::collections::HashSet`"
use std::collections::HashMap;

fn main() {
    let _map: HashMap<String, i32> = HashMap::new();
    let _set: HashSet<i32> = HashSet::new();
}
