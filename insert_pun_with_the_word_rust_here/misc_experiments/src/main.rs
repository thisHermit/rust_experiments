// Type your code here, or load an example.

// As of Rust 1.75, small functions are automatically
// marked as `#[inline]` so they will not show up in
// the output when compiling with optimisations. Use
// `#[no_mangle]` or `#[inline(never)]` to work around
// this issue.
// See https://github.com/compiler-explorer/compiler-explorer/issues/5939

// If you use `main()`, declare it as `pub` to see it in the output:
pub fn main() {
    let mut x = 1;

    if (x < 1) {
        x = 2;
    } else {
        x = 3;
    }
}
