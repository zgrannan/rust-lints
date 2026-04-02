# redundant_enum_variant

A [dylint](https://github.com/trailofbits/dylint) lint that detects non-pub
trait methods whose return type is an enum, where every implementation returns
the same single-field variant.

### What it does

For each non-public trait, inspects every method that returns an enum type.
If all implementations (including default bodies) always construct the same
variant, and that variant wraps exactly one field of type `T`, the lint warns
that the method could return `T` directly.

### Why is this bad?

An enum return type suggests that different implementations may produce
different variants. When they all produce the same one, the enum adds
indirection without value — callers must pattern-match on a variant that is
always the same.

### Example

```rust
enum Output {
    Text(String),
    Number(i64),
}

trait Render {
    fn render(&self) -> Output;
}

struct A;
impl Render for A {
    fn render(&self) -> Output {
        Output::Text("hello".into())
    }
}

struct B;
impl Render for B {
    fn render(&self) -> Output {
        Output::Text("world".into())
    }
}
```

Use instead:

```rust
trait Render {
    fn render(&self) -> String;
}
```

### Usage

Add the lint as a dylint library in your project's `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to>/rust-lints/redundant_enum_variant" },
]
```

Then run:

```sh
cargo dylint redundant_enum_variant
```
