This crate provides a proc macro that rewrites arithemtic operators `+,-,*` into their wrapping equivalents `wrapping_add, wrapping_sub, wrapping_mul` as well as their assigning versions `+=,-=,*=`.

The following function for example
````Rust
#[wrappit]
fn mix(a: u32, b: u32, c: [u32; 8]) -> u32 {
    let mut r = a + b;
    for u in c {
        r *= u;
    }
    r
}
````
is rewritten to
````Rust
fn mix(a: u32, b: u32, c: [u32; 8]) -> u32 {
    let mut r = a.wrapping_add(b);
    for u in c {
        r = r.wrapping_mul(u);
    }
    r
}