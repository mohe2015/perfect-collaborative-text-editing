# perfect-collaborative-text-editing

```rust
export RUSTFLAGS="-Zpolonius"

cargo fuzz run fuzz_target_1
cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-bd6a569f4d72f81386f4c3e7968bc01c9eff7bd0
cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-bd6a569f4d72f81386f4c3e7968bc01c9eff7bd0

cargo fuzz tmin --runs 255000 fuzz_target_1 fuzz/artifacts/fuzz_target_1/minimized-from-d9e610d1dc9662a9a28cf085ce4a195c8dc3da88
```

https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html