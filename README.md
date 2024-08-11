# perfect-collaborative-text-editing

```rust
export RUSTFLAGS="-Zpolonius"

cargo fuzz run fuzz_target_1
cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/minimized-from-d9dd7c9f723708057abf5d76d31591615d27e224
cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-bd6a569f4d72f81386f4c3e7968bc01c9eff7bd0

cargo fuzz tmin --runs 255000 fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-f4d526123671d989d1c8d1e65a77cd83077338d6
```

https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html