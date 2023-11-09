# Gear Ratio Calculator

A simple program to calculate the teeth on a gear given the other gear and a fixed transmission ratio, or the ratio from two gears.
Written in Rust using [egui](https://github.com/emilk/egui) with [eframe template](https://github.com/emilk/eframe_template).

## Building

You can run the app natively using `cargo run` or compile it to [WASM](https://en.wikipedia.org/wiki/WebAssembly) and use [Trunk](https://trunkrs.dev/) to view it as a web page.
To build for web target:
1. Install the required target with `rustup target add wasm32-unknown-unknown`.
2. Install Trunk with `cargo install --locked trunk`.
3. Run `trunk serve` to build and serve on `http://127.0.0.1:8080`. Trunk will rebuild automatically if you edit the project.
4. Open `http://127.0.0.1:8080/index.html in a browser. See the warning below.

