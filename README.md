# MOLEK-SYNTEZ Solitaire solver

Automated solution finder and executioner for MOLEK-SYNTEZ Solitaire minigame.
No command line interface is in place yet, you'll have to edit the code manually.

The bot works by finding any solutions by navigating the game state tree,
using the following heuristic (lower score preferred):
```rust
let mut score = 40;

for stack in &self.stacks {
    if stack.collapsed {
        ret -= 10
    } else {
        ret -= stack.highest_orderly_count()
    }
}

return score;
```

This method is used to find several solutions which are then optimized by
limited depth full tree exploration around the winning paths.
This has become pretty much redundant as since I improved the heuristic,
the optimization search takes longer than the moves it saves,
but I'm keeping it for testing more possible improvements later.

Get the game on [steam](https://store.steampowered.com/app/1168880/MOLEKSYNTEZ/)

## Mistake history

- My implementation of copying game states and passing past_matrices history was wrong,
  leading to loops and incorrect game states
- My heuristic score calculation was ineffective, reverted to preferring lowest move counts
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

## What I've learned

- That when I sometimes play fast and loose, I miss minor details,
  making me believe a more major fault is present.
- That I gave up way too early when I was really, really close to the correct heuristic.

## Requirements

I only ran this on linux so that's what I'm gonna list:

`apt-get install libxcb1 libxrandr2 libdbus-1-3 libxdo-dev`

`libxcb1`, `libxrandr2`, `libdbus-1-3` are for the `screenshots` crate

`libxdo-dev` is for the `enigo`


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