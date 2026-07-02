# MicroHs Rust Matrix

Updated: 2026-07-02

This file is a tracked working status artifact. Update it when parity,
performance, or benchmark classification changes.

## Worktree State

| item | state |
|---|---|
| branch | `microhs-rust` |
| upstream tracking | `origin/microhs-rust` |
| local commits ahead after this snapshot commit | 62 |
| runtime code baseline | generalized combinator app-reuse with `app_step!` reducer macro cleanup |
| dirty files after this snapshot commit | none expected |
| dirty work | none in tracked runtime files |
| matrix file | `MATRIX.md`, tracked from this snapshot |

## Verification Baseline

Last fully verified state: `app_step!` reducer macro cleanup.

| gate | status |
|---|---|
| `cargo test -p microhs-runtime --quiet` | passed before `app_step!` reducer macro cleanup commit; 27 tests |
| `cargo check -p microhs-runtime --lib --quiet` | passed before `app_step!` reducer macro cleanup commit |
| `cargo check -p microhs-runtime --bins --quiet` | passed before `app_step!` reducer macro cleanup commit |
| `cargo check --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed before `app_step!` reducer macro cleanup commit |
| `cargo build --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed before `app_step!` reducer macro cleanup commit |
| `node --check rust/microhs-runtime/js/host.mjs` | passed at `f8b9e1d5` |
| Node wasm core render smoke | passed at `9cbef47e`; host shim instantiated wasm, reduced `v8.4\n0\nI #5 @ }\n`, and rendered `5` |
| Node wasm dynamic `~I` JS FFI smoke | passed at `58280b6e`; path-loaded wasm reduced `IO.performIO ~I "return 40 + 2" @` and rendered `42` |
| Node wasm dynamic JS tag smoke | passed at `9cbef47e`; direct `~B`, `~S`, and `~J` smokes rendered `A`, `"hi"`, and `ForeignPtr#N` |
| Node wasm wrapper callback smoke | passed at `58280b6e`; byte-loaded wasm reduced `IO.performIO (IO.>>= (`II IO.return) (~IJ "return $0(35)"))` and rendered `35` |
| Node wasm wrapper tag coverage smoke | passed at `f8b9e1d5`; `II`, `UU`, `DD`, `FF`, `BB`, `SS`, `JJ`, and `PP` wrappers round-trip through JS and render expected values |
| Node wasm unsigned/high-bit JS smoke | passed at `f8b9e1d5`; direct `~U` rendered `4294967295`; direct and wrapper `~P` rendered `Ptr#2147483648` |
| Node wasm Response-source smoke | passed at `58280b6e`; `Response(bytes)` source reduced `~S "return 'hi'"` and rendered `"hi"` |
| `cargo fmt --all --check` | passed before `app_step!` reducer macro cleanup commit |
| `git diff --check` | passed before `app_step!` reducer macro cleanup commit |
| `make bin/mhsbench` | passed/up to date at `70eabf4d` |
| `cargo build --release --bin mhs-rust-bench --quiet` | passed before `app_step!` reducer macro cleanup commit |
| `cargo build --release --bin mhs-rust --quiet` | passed before generalized combinator app-reuse checkpoint commit |
| signed `i64::MIN` comb parser smoke | passed before checkpoint commit; Rust now parses the self-host compiler comb containing `##-9223372036854775808` |
| unbounded internal force budget smoke | passed before checkpoint commit; self-hosting no longer trips the old internal `10_000` WHNF cap |
| main-mode benchmark harness smoke | passed before checkpoint commit; Rust and C support `--mode main -- PROGRAM ARGS...`; main mode measures execution and validates external output instead of serializing the whole root graph |
| self-host compiler input generation | passed before checkpoint commit; `./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost.comb` produced a 647 KiB `v8.4` comb |
| self-host C main smoke | passed before checkpoint commit; updated `bin/mhsbench --mode main` produced `/tmp/mhs-selfhost-c-mainmode.comb`, a 647 KiB `v8.4` comb that parses under C WHNF smoke |
| lazy `readFile` CPP scan smoke | passed before checkpoint commit; temporary `hasLangCPP` reproducer now prints `False` under Rust like C, dropping the Rust one-shot time from ~433 ms before the fix to ~10 ms |
| self-host Rust main smoke | not yet passing; no-shim run no longer falsely invokes `cpphs`; latest full compile run at `4f6fa095` hit `timeout 600s` with exit `124` and produced no output comb at `/tmp/mhs-selfhost-rust-4f6fa095.comb` |
| `performio-apply-chain:200` benchmark | passed before checkpoint commit; Rust/C sinks match at `130000`, Rust `114,126` ns/iter vs C `123,407` ns/iter |
| direct Scott-pair selector perf probe | passed before checkpoint commit; current probe recognizes `U K (P x y)`/`U A (P x y)` after forcing the pair to WHNF; `hasLangCPP` proxy drops from `23,392` to `23,226` reductions/iter and measured `4,576,282` ns/iter Rust vs `635,079` ns/iter C, while self-host still exceeds the 1,200s cutoff |
| UTF-8 ASCII refill perf probe | passed before checkpoint commit; text `readFile` proxy improved to `3,134,640` ns/iter, `hasLangCPP` proxy to `4,395,391` ns/iter, and self-host `--help` main smoke to `123,325,990` ns/iter Rust vs `12,412,585` ns/iter C |
| direct `IO.return` bind perf probe | passed before checkpoint commit; self-host `--help` main smoke improved to `101,524,666` ns/iter Rust vs `10,021,074` ns/iter C; text proxies and common canaries stayed in-band |
| dynamic runtime profiler probe | passed before checkpoint commit; `mhs-rust-bench --profile --profile-top N` runs one extra profiled iteration after normal timing and reports head attempts/reductions, spine arity, heap-spine count, max arity, final node count, and sink |
| standalone runtime profiler smoke | passed before checkpoint commit; `target/release/mhs-rust --profile --profile-top 5 /tmp/mhs-rust-profile-smoke.comb` prints normal WHNF output on stdout and profile counters on stderr |
| self-host `--help` dynamic profile | passed before checkpoint commit; top reduction heads are `B` 62,065, `C` 36,472, `C'` 25,246, `C'B` 23,452, `P` 20,309; 120,563 heap spines and max arity 75 confirm the current bottleneck is reducer/spine throughput |
| uninitialized 16-slot inline spine perf probe | passed before checkpoint commit; self-host `--help` profile keeps the same 312,558 reductions and drops heap spines from 120,563 to 1,453; `MaybeUninit` avoids initializing unused inline slots, while `arith`/`zoo`/`data` canaries sink-match C under noisy timings |
| native Rust sampling profiler setup | installed `samply 0.13.1`; recording is currently kernel-blocked because `/proc/sys/kernel/perf_event_paranoid` is `2`; needs temporary `sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'` before `samply record --save-only ...` can collect samples |
| Rust/LLVM instrumentation profile | passed before profiling matrix commit; `llvm-tools-preview` installed, release bench rebuilt in `/tmp/mhs-rust-coverage-target` with `-C instrument-coverage -C force-frame-pointers=yes`, and self-host `--help` produced `/tmp/mhs-selfhost-help.profdata` plus `/tmp/mhs-selfhost-help-runtime-coverage.txt` |
| hot combinator app-reuse perf probe | passed before checkpoint commit; `B`, `C`, `C'`, `C'B`, and `P` now reuse the consumed redex app node for the final result app; self-host `--help` drops from `123,441,471` to `113,332,505` ns/iter and from `889,877` to `722,333` nodes after run with the same 312,558 reductions; rerun canaries sink-match C, with `arith`, `zoo`, and `data` better than prior matrix rows and `io`, `ffi-mem`, and `bfile-read` in the same noisy band |
| generalized combinator app-reuse perf probe | passed before checkpoint commit; final-app reuse now also covers pure combinator skeletons `U`, `S`, `S'`, `B'`, `Z`, `J`, `L`, `R`, `O`, partial `K2`/`K3`/`K4`, partial `C'B`, `Y`, `TAGn`, and `Tn`; self-host `--help` drops from `113,332,505` to `104,514,392` ns/iter and from `722,333` to `648,345` nodes after run with the same 312,558 reductions; rerun canaries sink-match C, with `arith`, `zoo`, and `data` better than prior matrix rows and `io`, `ffi-mem`, and `bfile-read` still in the same noisy band |
| `app_step!` reducer macro cleanup | passed before checkpoint commit; pure app-reuse reducer returns now use one local macro in `Program::step`; no intended behavior change; self-host `--help` canary still has sink `960903` and 312,558 WHNF steps, while raw timing was noisy on this machine |
| ignored IO action shortcut perf probe | passed before checkpoint commit; `io-chain`, `io-control-chain`, `argref-chain`, `ffi-chain`, `ffi-math-chain`, `ffi-const-chain`, `env-set-chain`, and `remove-missing-chain` improved with matching sinks; `ffi-mem-chain` and `bfile-read-chain` canaries stayed in the same band |
| direct lazyBind FFI-continuation perf probe | passed before checkpoint commit; against a clean `50e11139` temp worktree, `ffi-mem-chain` improved from ~1.31 ms to ~0.18 ms and `bfile-read-chain` improved from ~1.50 ms to ~0.34 ms with matching sinks; direct BFILE/env rows improved, while complex continuation canaries stayed in the same noisy band |
| reducer inline-spine perf probe | passed at `63d03844`; against `f8b9e1d5` temp build, current Rust improved `arith-chain`, `io-chain`, `ffi-chain`, `ffi-mem-chain`, and `bfile-read-chain` by roughly 4-13% |
| runtime primitive-cache perf probe | passed at `cf664e6f`; against `63d03844` matrix rows, current Rust improved `arith-chain`, `io-chain`, `ffi-chain`, `ffi-mem-chain`, and `bfile-read-chain` by roughly 2-7% |
| zero-arity FFI fast-path perf probe | passed at `646e75ab`; `ffi-chain` and `ffi-const-chain` improved with matching sinks; `io-chain` control row did not regress |
| unary math FFI fast-path perf probe | passed at `766a330a`; `ffi-math-chain:200` improved with matching sinks; `ffi-chain` and `ffi-mem-chain` controls remained in the same noise band |
| word-sized memory FFI return perf probe | passed at `c03a3441`; against a clean `766a330a` temp build, `ffi-wide-mem-chain:200` and `ffi-word-mem-chain:200` improved with matching sinks; `ffi-mem-chain:200` and `ffi-ptr-mem-chain:200` canaries stayed in the same noisy band |
| ArgRef IOArray cache perf/parity probe | passed at `681a3e12`; `argref-chain:200` improved with matching sinks, and `IO.getArgRef` now reuses the same argv IOArray like the C runtime |
| borrowed `strlen` perf probe | passed at `214d2e54`; `ffi-strcpy-chain`, `getenv-chain`, `getcwd-chain`, and `dir-read-chain` improved with matching sinks; non-string FFI canary stayed in the noisy baseline band |
| compiler-generated compressor write-path smokes | passed at `214d2e54`; temporary pure Haskell programs compiled with `bin/mhs -ilib -ihugs -o/tmp/...`; RLE/LZ77/BWT/LZMA compressor paths sink-match C |
| `array-chain` benchmark parity probe | passed at `70eabf4d`; scenario now uses the same IO/world-shaped array primitive contract as C and sink-matches |
| `md5-string-chain:20` smoke | passed at `10913996`; Rust/C sink `1320` |
| `md5Array` direct `.comb` smoke | passed at `10913996`; Rust result `176`, C sink `132` |
| `md5BFILE` direct `.comb` smoke | passed at `10913996`; Rust result `176`, C sink `132` |
| `getcwd-chain:20` smoke | passed at `76acd731`; Rust/C sink `1310` |
| `system` direct `.comb` smoke | passed at `76acd731`; Rust result `0`, C sink `130` |
| `chdir` direct `.comb` smoke | passed at `76acd731`; Rust result `0`, C sink `130` |
| `mkdir` direct `.comb` smoke | passed at `76acd731`; Rust result `0`, C sink `130` |
| `get_permissions` direct `.comb` smoke | passed at `76acd731`; Rust result `6`, C sink `130` |
| `set_permissions` direct `.comb` smoke | passed at `76acd731`; Rust result `0`, C sink `130` |
| `get_executable_path` direct `.comb` smoke | passed at `76acd731`; Rust path length result `54` |
| `dir-read-chain:20` smoke | passed at `ecc9b9f6`; Rust/C sink `1300` |
| directory EOF direct `.comb` smoke | passed at `ecc9b9f6`; Rust result `Ptr#0`, C sink `138` |
| `env-set-chain:20` smoke | passed at `64005968`; Rust/C sink `130` |
| `env-set-chain:200` benchmark | passed at `64005968`; Rust/C sink `130000` |
| `unsetenv` direct `.comb` smoke | passed at `64005968`; Rust result `0`, C sink `130` |
| `environ` direct `.comb` smoke | passed at `64005968`; Rust/C sink `131` |
| `errno-chain:20` smoke | passed at `cd81c3c8`; Rust/C sink `130` |
| `errno-chain:200` benchmark | passed at `cd81c3c8`; Rust/C sink `130000` |
| `strerror_r` direct `.comb` smoke | passed at `cd81c3c8`; Rust/C sink `131`; first byte `78` (`N`) |
| failed `remove` errno direct `.comb` smoke | passed at `cd81c3c8`; Rust result errno `2`, C sink `130` |
| `tmpname` direct `.comb` smoke | passed at `b53c633b`; Rust/C sink `131`; `TMPDIR=/tmp/mhs-rust-tmpname-oracle` |
| `getcpu` direct `.comb` smoke | passed at `b53c633b`; Rust/C sink `129` |
| `open`/`close` direct `.comb` smoke | passed at `2605435f`; Rust/C sink `130` |
| `open`/`add_fd`/`closeb` direct `.comb` smoke | passed at `2605435f`; Rust/C sink `129` |
| `gettimeofday` direct `.comb` smoke | passed at `2605435f`; Rust/C sink `130` |
| `O_NONBLOCK` direct `.comb` smoke | passed at `2605435f`; Rust/C sink `133` |
| `socket`/`close` direct `.comb` smoke | passed at `2605435f`; Rust/C sink `131` |
| `cargo test --quiet` | passed at `63702f10` |
| `cargo check -p microhs-runtime --lib --quiet` | passed at `63702f10` |
| `cargo check --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed at `63702f10` |
| `cargo fmt --all --check` | passed at `63702f10` |
| `git diff --check` | passed at `63702f10` |
| `make bin/mhsbench` | passed/up to date at `63702f10` |
| `cargo build --release --bin mhs-rust-bench --quiet` | passed at `63702f10` |
| `cargo build --release --bin mhs-rust --quiet` | passed at `63702f10` |
| runtimeFFI coverage script | passed at `63702f10`; missing `0` symbols |
| `mpz` init/get direct `.comb` smoke | passed at `63702f10`; Rust/C sink `131` |
| `mpz_add` direct `.comb` smoke | passed at `63702f10`; Rust/C sink `131` |
| `mpz_mul` negative direct `.comb` smoke | passed at `63702f10`; Rust/C sink `132` |
| `mpz_and` negative direct `.comb` smoke | passed at `63702f10`; Rust/C sink `130` |
| `mpz_tdiv_qr` negative direct `.comb` smoke | passed at `63702f10`; Rust/C sink `131` |
| `mpz_tstbit` negative direct `.comb` smoke | passed at `63702f10`; Rust/C sink `130` |
| `js_debug` helper direct `.comb` native smoke | passed at `63702f10`; release Rust reports `JavaScript FFI is not supported in this runtime`, not `UnknownFfi` |
| dynamic `~V` JS direct `.comb` native smoke | passed at `63702f10`; release Rust reports `JavaScript FFI is not supported in this runtime`, not parser/unknown-FFI failure |
| dynamic `~J` JS direct `.comb` native smoke | passed at `013ff522`; debug Rust reports `JavaScript FFI is not supported in this runtime`, not parser/unknown-FFI failure |
| JS wrapper direct `.comb` native smoke | passed at `013ff522`; debug Rust reports `JavaScript FFI is not supported in this runtime` through the wrapper host boundary |
| StablePtr callback trampoline API | passed at `e9ee217d`; compiles native and wasm; no committed test added |
| JS wrapper program-handle ABI | passed at `ac348a09`; compiles native and wasm; wrapper import now carries the owning Rust `Program` handle |
| JS wrapper callback API | passed at `5188c946`; compiles native and wasm; `Program::apply_js_wrapper` covers typed JS argument/result conversion behind the future wasm export |
| JS wrapper tag registry | passed at `ee1e0dfd`; compiles native and wasm; wrapper creation now passes a stable wrapper index instead of a temporary tag pointer |
| Rust wasm cdylib artifact | passed at `6949e200`; `cargo build --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` produces a `.wasm` module |
| Rust wasm embedding exports | passed at `d020fb56`; exports program create/free/reduce and wrapper invocation; wrapper invocation uses host `wargs` imports and returns status; original `try_borrow_mut` re-entry limitation fixed at `9cbef47e` |
| Rust wasm host shim | passed at `7a515e7b`; dependency-free Node ESM shim provides `mhs_js_*` imports, object registry, `argbuf`/`wargs`, and a basic instantiate/newProgram/reduce smoke |
| Rust wasm observable render | passed at `07710dab`; host shim can render reduced roots and assert direct dynamic JS FFI results |
| Rust wasm wrapper re-entry | passed at `9cbef47e`; active-program stack allows synchronous same-program wrapper callback re-entry and the Node wrapper smoke renders the callback argument |
| Rust wasm browser-loadable host shim | passed at `58280b6e`; static Node import removed; shim accepts path, file URL, bytes, `Response`, and browser-fetchable URLs |
| float serialization marker tests | passed at `74f7e283`; existing float/math assertions updated to C-compatible `.0` marker |
| `float64-chain:200` benchmark | passed at `74f7e283`; Rust/C sink `134000` |
| `float32-chain:200` benchmark | passed at `74f7e283`; Rust/C sink `135000` |
| C-compatible byte/string serializer rows | passed at `6438ec82`; `bytes-chain`, `cstring-pack`, `foreignptr-slice` sinks match |
| C-compatible graph serializer spacing | passed at `6438ec82`; `argref-chain`, `mvar-chain`, `weak-chain` sinks match |
| benchmark StablePtr harness reset | passed at `6438ec82`; `stableptr-chain` sink matches |

No tracked runtime behavior changes after the generalized combinator app-reuse checkpoint; the latest runtime edit is a local reducer macro cleanup.

## Tier Status

| tier / area | Rust status | wasm status | C oracle / benchmark status | notes |
|---|---|---|---|---|
| `.comb` parser and serializer | committed | checked | benchmarked | version `v8.4`, labels, application, literals |
| core reducer and combinators | committed | checked | benchmarked | core SK/zoo/data paths covered |
| numeric primops | committed | checked | benchmarked | Int, Int64, Float64, Float32 present; float serialization now keeps C-compatible decimal markers |
| ByteString primops | committed | checked | benchmarked | core bytes/list/UTF8 conversion paths present; byte-backed benchmark rows now sink-match C |
| arrays and mutable bytes | committed | checked | benchmarked | array benchmark rows now use the IO/world-shaped primitive contract that C implements |
| IO core, exceptions, masking | committed | checked | benchmarked | lightweight IO semantics, masking stubs, catch/raise covered; `IO.getArgRef` now reuses the argv IOArray like C |
| MVar / StablePtr / Weak / ForeignPtr | committed | checked | benchmarked | benchmark rows now sink-match C after serializer and C harness fixes |
| FFI constants/math/memory | committed | checked | benchmarked | dynamic FFI subset, errno constants, malloc/calloc/realloc/free, typed peek/poke |
| FFI strings/pointers | committed | checked | benchmarked | `strcpy`, `strlen`, `peekPtr/pokePtr`, `peekWord/pokeWord` |
| env/filesystem FFI | committed | checked | benchmarked | `getenv`, `setenv`, `unsetenv`, `environ`, `remove`, `getcwd`; native host behavior, errno recording, wasm fallback |
| memory BFILE | committed | checked | benchmarked | `openb_wr_mem`, `openb_rd_mem`, `get_mem`, `getb/putb/readb/writeb` |
| native file BFILE | committed | checked | benchmarked | `fopen`, `add_FILE`; text wrapper handled by UTF-8 BFILE row |
| UTF-8 BFILE transducer | committed | checked | benchmarked | `add_utf8` wrapper, modified-UTF-8 NUL decode verified |
| CRLF BFILE transducer | committed | checked | benchmarked | `add_crlf` wrapper, CRLF read collapse and LF write expansion |
| buffered BFILE transducer | committed | checked | benchmarked | `add_buf` wrapper, read/write buffering, line-buffer flush; raw benchmark uses runtime adapter order `ptr, size` |
| RLE BFILE transducers | committed | checked | partially benchmarked | `add_rle_compressor`, `add_rle_decompressor`; benchmark covers decompressor non-ASCII escape |
| base64 BFILE transducers | committed | checked | partially benchmarked | `add_base64_encoder`, `add_base64_decoder`; benchmark covers decoder whitespace and padding |
| LZ77 BFILE transducers | committed | checked | partially benchmarked | `add_lz77_compressor`, `add_lz77_decompressor`; benchmark covers decompressor literal block |
| BWT BFILE transducers | committed | checked | partially benchmarked | `add_bwt_compressor`, `add_bwt_decompressor`; benchmark covers decompressor single-byte block |
| LZMA BFILE transducers | committed | checked | partially benchmarked | `add_lzma_compressor`, `add_lzma_decompressor`; pure-Rust `lzma-sdk-rs`; benchmark covers decompressor with Rust-generated C-accepted `LZ2` payload |
| compression BFILE transducers | committed | checked | partially benchmarked | lz77, rle, bwt, base64, lzma implemented; compiler-generated write-path smokes now cover RLE/LZ77/BWT/LZMA compressors |
| MD5 runtime FFI | committed | checked | partially benchmarked | `md5String`, `md5Array`, `md5BFILE`; benchmark covers `md5String`, direct C-oracle smokes cover `md5Array` and `md5BFILE` first digest byte |
| directory/process FFI | committed | checked fallback | partially benchmarked | `system`, `chdir`, `mkdir`, `getcwd`, `get_permissions`, `set_permissions`, `get_executable_path`, `opendir`, `readdir`, `closedir`, `c_d_name`; host failures record errno |
| `Foreign.C.Error` FFI | committed | checked fallback | partially benchmarked | errno constants, `&errno`, `strerror_r`; direct smokes cover mutable errno, string lookup, and failed `remove` errno |
| temp-name / CPU-time FFI | committed | checked fallback | direct smokes | `tmpname` uses Unix `mkstemps`; `getcpu` uses `CLOCK_PROCESS_CPUTIME_ID` on Linux/Android; wasm fallbacks are explicit |
| fd/time/socket FFI | committed | checked fallback | direct smokes | `open`, `add_fd`, `close`, `gettimeofday`, socket constants, and thin Unix socket wrappers; guest pointer writes copy through temporary buffers |
| JS FFI hooks | committed, partial | builds wasm cdylib with browser-loadable host shim | native + Node wasm smokes | runtimeFFI helper names implemented; dynamic `~` calls support scalar, ByteString, and `JSVal` object handles through wasm host imports; direct wasm `~I`, `~B`, `~S`, `~J`, `~U`, and high-bit `~P` smokes render expected values; wrapper creation now allocates a StablePtr and passes the owning Rust `Program` handle plus stable wrapper index into the wasm host boundary; internal StablePtr-to-pointer and typed JS wrapper callback APIs exist; wasm exports invoke wrappers through host `wargs`; same-program wrapper callback smoke renders `35`; wrapper tag coverage smoke covers `II`, `UU`, `DD`, `FF`, `BB`, `SS`, `JJ`, and `PP`; host shim provides `mhs_js_*` and is browser-loadable for byte/Response/fetch sources; native reports explicit unsupported; high-level browser smoke still pending |
| mpz / imath FFI | committed | checked fallback | direct smokes | `new_mpz` plus 24 `mpz_*` symbols implemented with decimal-backed Rust `MpzValue`; add/mul/and/tdiv/tstbit smokes match C |
| Haskell compiler integration | keep Haskell | n/a | n/a | target is a Rust runtime/evaluator for `.comb`; compiler stays in Haskell |

## Benchmark Protocol

Unless stated otherwise:

| setting | value |
|---|---|
| Rust command | `target/release/mhs-rust-bench --scenario <name> --iters 1000 --warmup-iters 100 --c-mhsbench ./bin/mhsbench` |
| C command | invoked by Rust harness via `./bin/mhsbench` |
| default C mode | `whnf` |
| comparison rule | sink must match before using ratio as comparable |
| ratio | Rust ns/iter divided by C ns/iter |
| timing caveat | this machine may be running other work; trust sinks, reduction counts, and profile count distributions more than one-off raw timings |

## Benchmark Matrix

### Comparable Common Case

Rows in this section have matching Rust and C serialized sinks and are the
first performance targets: core reducer paths, normal numeric/bytes/array/IO
work, basic FFI, memory/string/pointer operations, file/BFILE basics, and common
runtime primitives.

| scenario | Rust ns/iter | C ns/iter | ratio | sink match |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 37,794 | 140,241 | 0.27 | yes |
| `arith-chain:200` | 51,520 | 104,942 | 0.49 | yes |
| `int64-chain:200` | 49,591 | 112,916 | 0.44 | yes |
| `float64-chain:200` | 61,633 | 112,001 | 0.55 | yes |
| `float32-chain:200` | 63,676 | 116,081 | 0.55 | yes |
| `bytes-chain:200` | 99,779 | 165,763 | 0.60 | yes |
| `cstring-pack:200` | 1,706 | 102,501 | 0.02 | yes |
| `foreignptr-slice:200` | 1,494 | 94,820 | 0.02 | yes |
| `unpack-chain:200` | 8,678 | 97,157 | 0.09 | yes |
| `fromutf8-chain:200` | 9,939 | 97,162 | 0.10 | yes |
| `array-chain:200` | 3,572 | 91,485 | 0.04 | yes |
| `io-chain:200` | 61,828 | 128,746 | 0.48 | yes |
| `io-array-chain:200` | 3,747 | 92,561 | 0.04 | yes |
| `io-bytes-chain:200` | 1,814 | 90,873 | 0.02 | yes |
| `io-control-chain:200` | 75,528 | 142,687 | 0.53 | yes |
| `performio-apply-chain:200` | 114,126 | 123,407 | 0.92 | yes |
| `argref-chain:200` | 55,481 | 128,597 | 0.43 | yes |
| `stdio-chain:200` | 143,397 | 112,014 | 1.28 | yes |
| `ffi-chain:200` | 64,427 | 158,747 | 0.41 | yes |
| `ffi-math-chain:200` | 82,002 | 170,476 | 0.48 | yes |
| `ffi-const-chain:200` | 76,276 | 200,473 | 0.38 | yes |
| `ffi-mem-chain:200` | 183,052 | 233,483 | 0.78 | yes |
| `ffi-wide-mem-chain:200` | 4,591,002 | 412,413 | 11.13 | yes |
| `ffi-word-mem-chain:200` | 3,996,918 | 373,408 | 10.70 | yes |
| `ffi-ptr-mem-chain:200` | 1,975,729 | 359,302 | 5.50 | yes |
| `ffi-strcpy-chain:200` | 4,358,649 | 401,217 | 10.86 | yes |
| `bfile-read-chain:200` | 301,920 | 271,403 | 1.11 | yes |
| `getenv-chain:200` | 930,970 | 370,071 | 2.52 | yes |
| `env-set-chain:200` | 675,467 | 646,157 | 1.05 | yes |
| `getcwd-chain:200` | 3,972,309 | 1,174,708 | 3.38 | yes |
| `file-read-close-chain:200` | 3,681,595 | 1,334,167 | 2.76 | yes |
| `utf8-bfile-read-chain:200` | 402,575 | 341,293 | 1.18 | yes |
| `crlf-bfile-read-chain:200` | 424,645 | 324,672 | 1.31 | yes |
| `buf-bfile-read-chain:200` | 1,618,560 | 360,791 | 4.49 | yes |
| `mvar-chain:200` | 20,784 | 117,743 | 0.18 | yes |
| `ptr-chain:200` | 80,302 | 107,577 | 0.75 | yes |
| `rnf-chain:200` | 75,782 | 3,801,194 | 0.02 | yes |
| `stableptr-chain:200` | 11,719 | 118,789 | 0.10 | yes |
| `weak-chain:200` | 20,320 | 115,161 | 0.18 | yes |
| `zoo-chain:300` | 158,832 | 127,845 | 1.24 | yes |
| `data-chain:300` | 165,659 | 132,097 | 1.25 | yes |

### Comparable Rare/Specialized

Rows in this section also have matching Rust and C serialized sinks, so the
ratio is meaningful. They are lower-priority for performance because they cover
specific libraries/codecs, diagnostic/error/failure paths, or less-common host
operations rather than the likely hot path for normal MicroHs programs.

| scenario | Rust ns/iter | C ns/iter | ratio | sink match |
|---|---:|---:|---:|---|
| `md5-string-chain:200` | 4,378,292 | 455,086 | 9.62 | yes |
| `errno-chain:200` | 4,126,808 | 446,975 | 9.23 | yes |
| `dir-read-chain:200` | 9,372,765 | 3,237,676 | 2.89 | yes |
| `remove-missing-chain:200` | 642,622 | 368,672 | 1.74 | yes |
| `base64-bfile-read-chain:200` | 1,685,450 | 341,881 | 4.93 | yes |
| `lz77-bfile-read-chain:200` | 2,555,758 | 1,119,364 | 2.28 | yes |
| `bwt-bfile-read-chain:200` | 1,607,342 | 366,929 | 4.38 | yes |
| `lzma-bfile-read-chain:200` | 1,932,381 | 507,415 | 3.81 | yes |
| `rle-bfile-read-chain:200` | 1,692,821 | 348,352 | 4.86 | yes |

### Self-Hosting Smoke And Performance

Self-hosting means running the Haskell MicroHs compiler comb under the runtime
to compile `MicroHs.Main` back to a `.comb`. The compiler stays Haskell; this
section only measures the C and Rust runtimes executing that compiler.

| check | C runtime | Rust runtime | status |
|---|---:|---:|---|
| compiler comb input | 647 KiB `/tmp/mhs-selfhost.comb`; generated by native `bin/mhs` | parses after signed-`i64::MIN` parser fix | input ready |
| `--help` main smoke | 10,810,951 ns/iter; sink `661902` | 104,514,392 ns/iter; sink `960903` | both run and print usage; not a full compile; refreshed with `--warmup-iters 1 --iters 3 --profile --profile-top 8`; macro-cleanup canary still sink-matches at `960903` and 312,558 WHNF steps, but raw timing was noisy |
| self-host compiler smoke | 56,387,928,187 ns/iter; output `/tmp/mhs-selfhost-c-refresh.comb` is a 647 KiB `v8.4` comb | latest full run at `4f6fa095` hit `timeout 600s` with exit `124`; no output comb at `/tmp/mhs-selfhost-rust-4f6fa095.comb` | not at parity |

### Self-Host Comb Static Profile

These counts are from `/tmp/mhs-selfhost.comb` with a lexer that respects
quoted strings and `$len` byte payloads.

| measure | count | implication |
|---|---:|---|
| file size | 661,784 bytes | small enough to inspect directly; large enough to expose compile-scale reducer costs |
| declared/defined labels | 3,226 / 3,226 | substantial sharing; update points and indirections matter |
| graph nodes after parse | 262,025 | one self-host iteration builds a large graph before doing source work |
| application nodes | 146,725 | app/spine traversal dominates static graph shape |
| primitive nodes | 104,562 | primitive dispatch cost matters |
| primitive nodes that are combinators | 98,233 | runtime hot path should focus on `B`, `C'`, `C`, `S'`, `C'B`, `A`, `O`, `U`, `S`, `B'`, `K`, `Z`, `P`, `T3` |
| `IO.*` primitive nodes | 167 | narrow IO fast paths help proxies but cannot explain full self-host cost alone |
| `A.*` primitive nodes | 10 | generic IOArray fast paths are unlikely to be a major compile-scale lever |
| FFI nodes | 205 | FFI coverage/per-call cost is not the main self-host bottleneck |
| byte literals | 2,319 literals / 32,694 bytes total | source/help strings are present but not the dominant graph mass |
| top shared labels | `_128` `(U K)` refs 1,004; `_0` `(U (K K2))` refs 900; `_6` `B` refs 778; `_203` `((C' Y) ((C'B P) (C'B O)))` refs 707 | repeated selector/combinator skeletons should guide the next reducer specialization |

### Self-Host Dynamic Profile

These counts are from:

`timeout 60s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 --profile --profile-top 20 -- ./bin/mhs --help`

In the bench harness, `--profile` runs one extra profiled iteration after normal
timing, so the normal benchmark output stays comparable. The standalone
`mhs-rust --profile --profile-top N` path prints the same counters to stderr
while leaving the rendered result on stdout.

| measure | count | implication |
|---|---:|---|
| profiled iteration time | 137.686 ms | instrumentation overhead is separate from normal timing |
| reductions | 312,558 | same reduction count as the normal `--help` smoke |
| step attempts | 338,053 | only 28,358 attempts do not reduce; most work is real reduction |
| successful step heads | 309,695 | optimized multi-reduction steps account for the gap to total reductions |
| nodes after run | 648,345 | generalized combinator app reuse removes another 73,988 transient app nodes from this proxy; the run still grows far beyond the parsed input |
| heap spines | 1,453 | increasing inline storage to 16 covers almost all hot arities without heap allocation |
| max spine arity | 75 | long application spines are present even in the short `--help` path |

Top dynamic reduction heads:

| rank | head | reductions |
|---:|---|---:|
| 1 | `Prim:B` | 62,065 |
| 2 | `Prim:C` | 36,472 |
| 3 | `Prim:C'` | 25,246 |
| 4 | `Prim:C'B` | 23,452 |
| 5 | `Prim:P` | 20,309 |
| 6 | `Prim:K` | 14,202 |
| 7 | `Prim:S` | 13,980 |
| 8 | `Prim:Z` | 13,932 |
| 9 | `Prim:O` | 8,986 |
| 10 | `Prim:A` | 8,863 |
| 11 | `Prim:S'` | 8,484 |
| 12 | `Prim:IO.>>=` | 8,397 |
| 13 | `Prim:B'` | 8,350 |
| 14 | `Prim:Y` | 6,174 |
| 15 | `Prim:IO.>>` | 5,716 |
| 16 | `Prim:IO.return` | 5,602 |
| 17 | `Prim:T3` | 5,569 |
| 18 | `Prim:R` | 5,562 |
| 19 | `Prim:K3` | 5,560 |
| 20 | `Prim:U` | 2,985 |

Spine arity hotspots:

| arity | attempts |
|---:|---:|
| 7 | 61,669 |
| 8 | 42,648 |
| 9 | 31,206 |
| 6 | 31,059 |
| 5 | 30,649 |
| 12 | 25,392 |
| 0 | 28,310 |
| 10 | 17,118 |
| 14 | 14,070 |
| 13 | 11,268 |

Inline spine A/B notes:

| variant | self-host profile heap spines | self-host `--help` Rust ns/iter | canary note |
|---|---:|---:|---|
| committed baseline, 8 initialized slots | 120,563 | noisy; prior refreshed row was 123,570,659 | old state initialized unused inline slots with sentinels |
| 8 uninitialized slots | 120,563 | 140,269,254 | best low-arity `arith` canary in this run, but no heap-spine reduction |
| 12 uninitialized slots | 38,222 | 130,698,554 | worse than 16 on profile and not better on canaries |
| 16 uninitialized slots | 1,453 | 123,441,471 | selected; best self-host `--help` result in this A/B, with `arith` still faster than C but lower-arity timing not clearly improved |

### Rust/LLVM Hot-Count Profile

Native `samply` sampling is installed but currently blocked by Linux perf
permissions (`perf_event_paranoid=2`). Until that is lowered to `1`, Rust/LLVM
coverage instrumentation gives execution-frequency data but not CPU sample time.

Coverage build:

`CARGO_TARGET_DIR=/tmp/mhs-rust-coverage-target CARGO_PROFILE_RELEASE_DEBUG=1 RUSTFLAGS='-C instrument-coverage -C force-frame-pointers=yes' cargo build --release --bin mhs-rust-bench --quiet`

Profiled run:

`LLVM_PROFILE_FILE='/tmp/mhs-selfhost-help-%m-%p.profraw' /tmp/mhs-rust-coverage-target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 1 --iters 10 -- ./bin/mhs --help`

Hot runtime line counts over 10 instrumented `--help` iterations:

| area | line/count signal | implication |
|---|---:|---|
| `resolve` | 65.1M function entries; 66.9M node lookups | indirection chasing and node classification are the highest-frequency runtime operations |
| spine walk | 32.3M `Node::App` loop checks; 28.6M arg/fun resolves | long-spine traversal is still central even after heap-spine spills dropped |
| app rebuild | 17.9M extra-argument rethread iterations in `apply_reduction_spine` | post-reduction spine rebuilding is a major hot path and likely next optimization target |
| allocation | 6.87M `push_node` calls; 6.74M `app` calls | the `--help` path allocates many transient application nodes |
| reducer loop | 3.71M `step` entries; 3.40M reduction results | most loop trips reduce, so dispatch and rewrite mechanics dominate over failed attempts |
| primitive dispatch | hot fall-through through the `Prim` match cascade | avoid cloning/matching primitive strings in the reducer if a small, safe representation change pays off |

### Comment

| topic | current theory |
|---|---|
| Rust/C performance gap | The ignored-action, direct lazyBind, performIO extra-spine, direct Scott-pair selector, UTF-8 ASCII refill, direct `IO.return` bind, uninitialized 16-slot inline spine, hot/generalized combinator app-reuse probes, dynamic runtime counters, and Rust/LLVM hot-count profile confirm the gap is mostly evaluator overhead, not parity noise. Simple `IO.>>`, direct FFI, unary result-fed FFI/BFILE chains, lazy performIO application, ASCII text reads, trivial returned binds, and most self-host `--help` spine traversals are now close to C or materially better than before. The static, dynamic, and LLVM profiles point away from IO/array/FFI as the main full-compiler blocker. Because this machine may be busy, raw timings are treated as indicative only; the stronger signal is the count distribution. The active theory is compile-scale reducer/runtime throughput: `resolve`, spine traversal, remaining app allocation/update mechanics, primitive dispatch, and IO-bind skeletons. Pure final-app combinator allocation is now broadly addressed and factored through `app_step!`; recent lazy-argument and broad IO app-reuse probes did not hold up, so remaining candidates are primitive-name dispatch, repeated `resolve` traffic, and more targeted IO-specific rewrites that do not reintroduce indirection blow-ups. |

### Compiler-Generated Compressor Write Smokes

Rows in this section are generated from temporary pure Haskell programs compiled by the C MicroHs compiler with `bin/mhs -ilib -ihugs -o/tmp/<name>.comb /tmp/<name>.hs`. They exercise high-level `System.Compress` compressor write paths and sink-match C, but are not built-in `mhs-rust-bench --scenario` names yet.

| smoke | Rust ns/iter | C ns/iter | ratio | sink match |
|---|---:|---:|---:|---|
| `length (compressRLE "AAAAAAAAAAAAAAAA")` | 528,586 | 287,581 | 1.84 | yes |
| `length (compressLZ77 "AAAAAAAAAAAAAAAA")` | 551,016 | 286,730 | 1.92 | yes |
| `length (compressBWT "AAAAAAAAAAAAAAAA")` | 579,428 | 288,223 | 2.01 | yes |
| `length (compressLZMA "AAAAAAAAAAAAAAAA")` | 620,141 | 678,996 | 0.91 | yes |

## Immediate Next Items

| item | reason | status |
|---|---|---|
| compression BFILE write-path benchmarks | decompressor rows existed; compiler-generated high-level compressor smokes now cover RLE/LZ77/BWT/LZMA write paths | partial done; built-in repeat scenarios still pending |
| MD5 broader coverage | committed runtime covers all three FFI names; only `md5String` has a repeat benchmark | pending full high-level `System.IO.MD5` test once the compiler binary is available |
| directory iteration FFI | committed runtime covers `opendir`, `readdir`, `closedir`, `c_d_name` | pending full high-level `System.Directory` test once the compiler binary is available |
| self-hosting parity | C runtime can execute the compiler comb and produce a new compiler comb | Rust main-mode harness exists and lazy file IO now matches the C CPP scan smoke; latest full-compile run at `4f6fa095` still hit the 600s timeout with no output comb; inline spine work removed almost all `--help` heap-spine spills, and generalized combinator app reuse cuts self-host `--help` node growth by ~27%, but full self-host parity still needs more reducer throughput work |
| high-level environment tests | runtime now covers `getenv`, `setenv`, `unsetenv`, `environ`; benchmark covers mutation plus lookup | pending full high-level `System.Environment` test once the compiler binary is available |
| high-level errno/error tests | runtime now covers errno constants, `&errno`, `strerror_r`, and errno recording for implemented host failures | pending full high-level `Foreign.C.Error` / `throwErrnoIf*` tests once the compiler binary is available |
| high-level temp/CPU tests | runtime now covers `tmpname` and `getcpu`; direct smokes cover the raw FFI actions only | pending full high-level `System.IO.openTmpFile` / `System.CPUTime` tests once the compiler binary is available |
| commit runtime parity checkpoint | `63702f10 Add Rust mpz and JS FFI coverage` | done |
| JS full parity | runtimeFFI missing-symbol coverage is zero; `JSVal` object result/argument handles, wrapper creation boundary, owning-program handle plumbing, wrapper tag registry, internal StablePtr callback trampoline, typed JS wrapper callback API, real Rust `.wasm` build artifact, wasm embedding exports, browser-loadable JS host shim, direct wasm dynamic JS FFI smoke, same-program wrapper callback smoke, and broad wrapper tag coverage are in place, but high-level `foreign import javascript` browser parity is not complete; `.combffi` probing did not expose enough JavaScript import metadata for a runtime-only bridge | pending compiler-emitted metadata or generated glue path |
| performance phase | sixteen performance checkpoints plus one benchmark parity checkpoint plus static, dynamic, and Rust/LLVM profile checkpoints committed; numeric benchmark matrix rows now all sink-match C; rows are split into common-case and rare/specialized; common-operation targets should shift from narrow IO/FFI probes to `resolve`, primitive dispatch, remaining spine traversal, app allocation/update mechanics, and IO-specific reductions where Rust is still well behind C at self-host scale | active |
| rejected performance probes | `KnownFfi` enum/symbol specialization, single-pass BFILE read dispatch, direct unit-return FFI pairing, direct Unix `getcwd` into guest allocation, head-node clone collapse in `step`, primitive-node cache, broad fixed-primitive borrowed classifier, pre-clone `IO.>>` branch, broad borrowed C-string host paths, borrowed `md5String`, over-broad pointer FFI pre-dispatch, generic-arm `peekPtr`/`pokePtr` direct returns, zero-arity FFI candidate guard, direct zero-arity FFI under `IO.>>`, direct FFI shortcut under `IO.lazyBind`, borrowed fixed-size peeks, runtime handle-table free lists, shared `Rc<str>` node symbols, `IO.>>=` direct array-action execution, the `IO.return`/`K` continuation collapse, broad fast-combinator pre-dispatch, skipping outer-app rethreading after reduction, always-inlining the hot small reducer helpers, 12-slot inline spine storage, 8-slot uninitialized-only spine storage, lazy argument resolution in `spine`, and broad IO final-app reuse were measured and reverted or not selected because important end-to-end rows were neutral or slower | done, do not reapply blindly |
| keep compiler Haskell | scope is C runtime/evaluator rewrite; Haskell compiler remains authoritative `.comb` producer | ongoing |
