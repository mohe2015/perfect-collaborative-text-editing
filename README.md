# perfect-collaborative-text-editing

```rust
export RUSTFLAGS="-Zpolonius"
export RUST_BACKTRACE=1

cargo fuzz run --jobs 8 fuzz_target_1 -- -max_total_time=60 
cargo fuzz run fuzz_target_2 fuzz/artifacts/fuzz_target_2/minimized-from-738ea8e0d431702766c7ee86257dd224f0677b57
cargo fuzz tmin --runs 255000 fuzz_target_2 fuzz/artifacts/fuzz_target_2/crash-465e4f60b74c371055104678d4387f66af744d81

rm fuzz/coverage/fuzz_target_1/coverage.profdata
cargo fuzz coverage fuzz_target_1
~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_target_1 --format=html --output-dir=target/cov --ignore-filename-regex=.cargo/registry/src -instr-profile=fuzz/coverage/fuzz_target_1/coverage.profdata
firefox target/cov/index.html
```

https://altsysrq.github.io/proptest-book/proptest/vs-quickcheck.html