# unicorn-hat-hd

Rust library for interacting with the Pimoroni Unicorn HAT HD.

## Documentation

The docs can be found online at [docs.rs](https://docs.rs/unicorn_hat_hd/), or be built using `cargo doc`.

## Example

Add `unicorn_hat_hd` to your `Cargo.toml`.

```toml
[dependencies]
unicorn_hat_hd = "0.1"
```

Add `unicorn_hat_hd` to your crate root.

```rust
extern crate unicorn_hat_hd;
```

Create a default `UnicornHatHd`, and start setting some pixels.

```rust
use unicorn_hat_hd::UnicornHatHd;

pub fn main() {
    let mut hat_hd = UnicornHatHd::default();
    loop {
        for y in 0..16 {
            for x in 0..16 {
                hat_hd.set_pixel(x, y, 255, 255, 255);
                hat_hd.display().unwrap();
                hat_hd.set_pixel(x, y, 0, 0, 0);
            }
        }
    }
}
```

## Copyright and license

Copyright (c) 2017 Jacob Helwig. Released under the [BSD license](LICENSE).
