# Redistricting
For Bevy Jam 1: https://itch.io/jam/bevy-jam-1

Play it on itch.io: https://the-nacho.itch.io/redistricting

Licensed under the dual MIT / Apache-2.0 license

## Future features
* Saving and loading
* Colors other than just red and blue
* Campaign mode where you get to "pass a law" (get a power-up) between each level
* Sound
* UI prettyness
* Level editor
* Ability to see level seeds and load levels based seeds
* Daily puzzle with seed based on the date
* Undo
* Fill tool
* Ability to lock districts

## Building for web
### Prerequisites
* `rustup target install wasm32-unknown-unknown`
* `cargo install wasm-bindgen-cli`
### Build
1. `cargo build --release --target wasm32-unknown-unknown`
1. `wasm-bindgen --out-dir out --target web target/wasm32-unknown-unknown/release/bevy-jam-01.wasm`
1. `cp index.html out`
1. `cp -r assets out`