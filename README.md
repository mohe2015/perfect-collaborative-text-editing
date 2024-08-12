# perfect-collaborative-text-editing

```rust
export RUSTFLAGS="-Zpolonius"
export RUST_BACKTRACE=1

cargo fuzz run fuzz_target_1
cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/minimized-from-d9dd7c9f723708057abf5d76d31591615d27e224
cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-bd6a569f4d72f81386f4c3e7968bc01c9eff7bd0

cargo fuzz tmin --runs 255000 fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-eb68b72d2fd8c1a93dc55d1cd51ada454504f617
```

https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html