# perfect-collaborative-text-editing

```rust
export RUSTFLAGS="-Zpolonius"
export RUST_BACKTRACE=1

cargo fuzz run --jobs 8 fuzz_target_1 -- -max_total_time=60 
cargo fuzz run fuzz_target_1 fuzz/artifacts/fuzz_target_1/minimized-from-d9dd7c9f723708057abf5d76d31591615d27e224
cargo fuzz tmin fuzz_target_1 fuzz/artifacts/fuzz_target_1/crash-bd6a569f4d72f81386f4c3e7968bc01c9eff7bd0

cargo fuzz tmin --runs 255000 fuzz_target_2 fuzz/artifacts/fuzz_target_2/crash-92b8896cc3454111624b8be0ebd7c9d289068931

rm fuzz/coverage/fuzz_target_1/coverage.profdata
cargo fuzz coverage fuzz_target_1
~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_target_1 --format=html --output-dir=target/cov --ignore-filename-regex=.cargo/registry/src -instr-profile=fuzz/coverage/fuzz_target_1/coverage.profdata
firefox target/cov/index.html
```

https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html