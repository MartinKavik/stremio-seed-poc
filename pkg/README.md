# Stremio seed example

- If you don't have Rust and cargo-make installed, [Download it](https://www.rust-lang.org/tools/install), and run the following commands:

`rustup update`

`rustup target add wasm32-unknown-unknown`

`cargo install --force cargo-make`

Run `cargo make all` in a terminal to build the app, and `cargo make serve` to start a dev server
on `127.0.0.0:8000`.

