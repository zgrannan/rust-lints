fn main() {
    let r: Result<i32, &str> = Ok(1);

    // Should warn: .ok().is_some()
    let _ = r.ok().is_some();

    // Should NOT warn: .is_ok() directly
    let _ = r.is_ok();

    // Should NOT warn: .ok().is_none() — different pattern, no equivalent one-liner
    let _ = r.ok().is_none();

    // Should NOT warn: .ok() used separately from .is_some()
    let v: Option<i32> = r.ok();
    let _ = v.is_some();
}
