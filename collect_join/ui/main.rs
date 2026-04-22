fn main() {
    let words = vec!["hello", "world"];

    // Should warn: collect::<Vec<_>> then join
    let _ = words.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(", ");

    // Should warn: collect::<Vec<String>> then join
    let _ = words.iter().map(|w| w.to_string()).collect::<Vec<String>>().join(" ");

    // Should NOT warn: join on a pre-existing Vec (no collect)
    let v: Vec<String> = vec!["a".to_string(), "b".to_string()];
    let _ = v.join("-");

    // Should NOT warn: join on a slice
    let _ = words.join(", ");
}
