// Should warn: path has 4 segments (outside of use)
fn long_type() -> std::collections::hash_map::HashMap<i32, i32> {
    Default::default()
}

// Should NOT warn: inside a use statement
use std::collections::hash_map::HashMap;

// Should NOT warn: only 3 segments
fn short_type() -> std::collections::HashMap<i32, i32> {
    Default::default()
}

fn main() {
    let _ = long_type();
    let _: HashMap<i32, i32> = Default::default();
    let _ = short_type();
}
