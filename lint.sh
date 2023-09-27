cargo +nightly fmt
cargo +nightly clippy --all-targets --all-features --fix --allow-staged --allow-dirty -- -D warnings
