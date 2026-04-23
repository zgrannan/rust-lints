# empty_if

A [dylint](https://github.com/trailofbits/dylint) lint that warns on `if`
expressions whose then-block is empty.

### What it does

Checks for `if` expressions where the then-block contains no statements.

### Why is this bad?

An empty then-block is always a code smell:

- `if cond {}` with no else is dead code and should be removed.
- `if cond {} else { body }` should be written as `if !cond { body }`, which
  makes the condition match the action and is easier to read.

`if let` patterns and let-chains are excluded because negating them is
non-trivial and is not flagged.

### Example

```rust
if x.is_some() {} else {
    handle_none();
}
```

Use instead:

```rust
if x.is_none() {
    handle_none();
}
```

### Usage

Add the lint as a dylint library in your project's `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to>/rust-lints/empty_if" },
]
```

Then run:

```sh
cargo dylint empty_if
```
