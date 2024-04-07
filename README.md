# MOLEK-SYNTEZ Solitaire solver

So far this is just a basic playable command line version of the solitaire.
It actually works now, there were just some oversights that caused infinite loops but the concept was correct.
Such a me mistake. Anyway, next I'll see how hard/easy it is to grab screenshots, determine positions, emulate clicks, etc.

Me trying to make more involved heuristics on anything but my first attempt also seems doomed to failure.
I tried this approach to calculate the score of a stack and then summing them to get the score of a matrix:

```rust
if self.collapsed {
    return 20
} else if self.cheated {
    return 0;
} else if self.cards.is_empty() {
    return 20;
} else {
    return self.highest_orderly_count() + self.cards.len();
}
```

This naturally led to even poorer performance, so I reverted this and instead decided to focus
on finding multiple solutions and shortening the existing ones.

Get the game on [steam](https://store.steampowered.com/app/1168880/MOLEKSYNTEZ/)

## Requirements
I only ran this on linux so that's what I'm gonna list. TL;DR:

`apt-get install libxcb1 libxrandr2 libdbus-1-3 libxdo-dev`

`libxcb1`, `libxrandr2`, `libdbus-1-3` are for screenshots, `libxdo-dev` is for enigo (robot-like mouse control)


## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.