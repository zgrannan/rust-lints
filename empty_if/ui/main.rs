// edition:2024

fn main() {
    let x: Option<i32> = Some(1);

    // Should warn: empty then-block with else
    if x.is_some() {} else {
        println!("none");
    }

    // Should warn: empty then-block with else-if chain
    if x.is_some() {} else if x.is_none() {
        println!("none");
    }

    // Should warn: empty then-block, no else (dead code)
    if x.is_some() {}

    // Should NOT warn: non-empty then-block
    if x.is_some() {
        println!("some");
    } else {
        println!("none");
    }

    // Should NOT warn: comment in the block acts as a placeholder
    if x.is_some() {
        // TODO: handle this case
    } else {
        println!("none");
    }

    // Should NOT warn: if let — negation is non-trivial
    if let Some(_) = x {} else {
        println!("none");
    }

    // Should NOT warn: let chain — negation is non-trivial
    let y: Option<i32> = Some(2);
    if x.is_some() && let Some(_) = y {} else {
        println!("nope");
    }
}
