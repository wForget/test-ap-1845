## Reproduction steps

On Linux x86-64, unpack async-profiler `4.6-55f7f0e-linux-x64` in the project directory. Compile [rust_preload_probe.rs](rust_preload_probe.rs) and run it with the profiler preloaded:

```sh
rustup toolchain install nightly --profile minimal
rustc +nightly --edition=2021 -C opt-level=2 -C debuginfo=2 \
  -C force-unwind-tables=yes rust_preload_probe.rs
LD_PRELOAD="$PWD/async-profiler-4.6-55f7f0e-linux-x64/lib/libasyncProfiler.so" \
  ASPROF_COMMAND='start,event=cpu,interval=10ms,wall=100ms,nativemem,cstack=dwarf,file=profiler.jfr' \
  ./rust_preload_probe
```

## Root cause

In the original crash, wall sampling interrupted CPU sampling on the same Rust worker. The nested handlers exhausted its 8 KiB alternate signal stack, causing `SIGSEGV` in `Profiler::getNativeTrace`.
