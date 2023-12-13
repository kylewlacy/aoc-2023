# Kyle's Advent of Code 2023 solutions

This repo contains my solutions for [Advent of Code 2023](https://adventofcode.com/2023). I used Rust for all my implementations, where each day and part are separate crates (parts 1 and 2 are usually similar, but I didn't find it valuable to try and share code between the two, so I usually started part 2 by copying the implementation from part 1). All solutions use stable Rust except those with a `rust-toolchain.toml` file (when using Rustup, this file should automatically alert you if you don't have the right Rust toolchain installed).

## Setup and usage

I  [cargo-generate](https://github.com/cargo-generate/cargo-generate) with a custom template within this repo to speed up the process of scaffolding each new part. To set up a new day or part, run this in the root of the repo after installing `cargo-generate`:

```sh-session
$ cargo generate --path template -n day${X}-part${Y}
```

Each implementation reads the puzzle input from stdin. The example input from Advent of Code is included, but the real puzzle input is excluded (for convenience, this can be saved under `fixtures/input.txt` under any day, and it will be excluded by the `.gitignore` rules).

To output the solution for the example from day 1 part 1, for instance:

```sh-session
$ cd day01-part1
$ cargo run --release < fixtures/example.txt
142
```

## Implementation notes

My goal was to just get the solution for each day as quick as possible. So, code quality and robustness generally take a backseat. While I tried to keep my code fairly idiomatic (using iterators, using enums and structs to represent values, `?`-based error handling), I wasn't really strict with following any particular principles. That's also why I didn't focus on writing tests or comments, like you often see for other Advent of Code solutions.

I tried to solve each puzzle blind, although I was open to using any existing crates from <https://crates.io/>. I also occasionally bounced ideas around with my wife who was also doing Advent of Code, and only fell back to looking up hints online if I wasn't able to solve a puzzle after 24 hours (noted in the commit messages and as a comment in the solutions).
