# MicroHs Rust Matrix

Updated: 2026-07-02

This file is a tracked working status artifact. Update it when parity,
performance, or benchmark classification changes.

## Worktree State

| item | state |
|---|---|
| branch | `microhs-rust` |
| upstream tracking | `origin/microhs-rust` |
| local commits ahead after this snapshot commit | 47 |
| runtime code baseline | ignored IO action shortcut checkpoint |
| dirty files after this snapshot commit | none expected |
| dirty work | none in tracked runtime files |
| matrix file | `MATRIX.md`, tracked from this snapshot |

## Verification Baseline

Last fully verified state: ignored IO action shortcut checkpoint.

| gate | status |
|---|---|
| `cargo test --quiet` | passed at `70eabf4d` |
| `cargo check -p microhs-runtime --lib --quiet` | passed at `70eabf4d` |
| `cargo check -p microhs-runtime --bins --quiet` | passed at `70eabf4d` |
| `cargo check --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed at `70eabf4d` |
| `cargo build --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed at `70eabf4d`; produced `target/wasm32-unknown-unknown/debug/microhs_runtime.wasm` (5.0 MiB) |
| `node --check rust/microhs-runtime/js/host.mjs` | passed at `f8b9e1d5` |
| Node wasm core render smoke | passed at `9cbef47e`; host shim instantiated wasm, reduced `v8.4\n0\nI #5 @ }\n`, and rendered `5` |
| Node wasm dynamic `~I` JS FFI smoke | passed at `58280b6e`; path-loaded wasm reduced `IO.performIO ~I "return 40 + 2" @` and rendered `42` |
| Node wasm dynamic JS tag smoke | passed at `9cbef47e`; direct `~B`, `~S`, and `~J` smokes rendered `A`, `"hi"`, and `ForeignPtr#N` |
| Node wasm wrapper callback smoke | passed at `58280b6e`; byte-loaded wasm reduced `IO.performIO (IO.>>= (`II IO.return) (~IJ "return $0(35)"))` and rendered `35` |
| Node wasm wrapper tag coverage smoke | passed at `f8b9e1d5`; `II`, `UU`, `DD`, `FF`, `BB`, `SS`, `JJ`, and `PP` wrappers round-trip through JS and render expected values |
| Node wasm unsigned/high-bit JS smoke | passed at `f8b9e1d5`; direct `~U` rendered `4294967295`; direct and wrapper `~P` rendered `Ptr#2147483648` |
| Node wasm Response-source smoke | passed at `58280b6e`; `Response(bytes)` source reduced `~S "return 'hi'"` and rendered `"hi"` |
| `cargo fmt --all --check` | passed at `70eabf4d` |
| `git diff --check` | passed at `70eabf4d` |
| `make bin/mhsbench` | passed/up to date at `70eabf4d` |
| `cargo build --release --bin mhs-rust-bench --quiet` | passed at `70eabf4d` |
| `cargo build --release --bin mhs-rust --quiet` | passed at `70eabf4d` |
| ignored IO action shortcut perf probe | passed before checkpoint commit; `io-chain`, `io-control-chain`, `argref-chain`, `ffi-chain`, `ffi-math-chain`, `ffi-const-chain`, `env-set-chain`, and `remove-missing-chain` improved with matching sinks; `ffi-mem-chain` and `bfile-read-chain` canaries stayed in the same band |
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

No tracked runtime changes after the `array-chain` benchmark parity commit.

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
| C mode | `whnf` |
| comparison rule | sink must match before using ratio as comparable |
| ratio | Rust ns/iter divided by C ns/iter |

## Benchmark Matrix

### Comparable Common Case

Rows in this section have matching Rust and C serialized sinks and are the
first performance targets: core reducer paths, normal numeric/bytes/array/IO
work, basic FFI, memory/string/pointer operations, file/BFILE basics, and common
runtime primitives.

| scenario | Rust ns/iter | C ns/iter | ratio | sink match |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 39,040 | 136,138 | 0.29 | yes |
| `arith-chain:200` | 46,687 | 98,981 | 0.47 | yes |
| `int64-chain:200` | 49,591 | 112,916 | 0.44 | yes |
| `float64-chain:200` | 61,633 | 112,001 | 0.55 | yes |
| `float32-chain:200` | 63,676 | 116,081 | 0.55 | yes |
| `bytes-chain:200` | 99,779 | 165,763 | 0.60 | yes |
| `cstring-pack:200` | 1,706 | 102,501 | 0.02 | yes |
| `foreignptr-slice:200` | 1,494 | 94,820 | 0.02 | yes |
| `unpack-chain:200` | 8,678 | 97,157 | 0.09 | yes |
| `fromutf8-chain:200` | 9,939 | 97,162 | 0.10 | yes |
| `array-chain:200` | 3,572 | 91,485 | 0.04 | yes |
| `io-chain:200` | 60,782 | 131,232 | 0.46 | yes |
| `io-array-chain:200` | 3,747 | 92,561 | 0.04 | yes |
| `io-bytes-chain:200` | 1,814 | 90,873 | 0.02 | yes |
| `io-control-chain:200` | 75,528 | 142,687 | 0.53 | yes |
| `argref-chain:200` | 55,481 | 128,597 | 0.43 | yes |
| `stdio-chain:200` | 143,397 | 112,014 | 1.28 | yes |
| `ffi-chain:200` | 64,427 | 158,747 | 0.41 | yes |
| `ffi-math-chain:200` | 82,002 | 170,476 | 0.48 | yes |
| `ffi-const-chain:200` | 76,276 | 200,473 | 0.38 | yes |
| `ffi-mem-chain:200` | 1,224,237 | 239,483 | 5.11 | yes |
| `ffi-wide-mem-chain:200` | 3,780,602 | 382,665 | 9.88 | yes |
| `ffi-word-mem-chain:200` | 3,753,346 | 403,997 | 9.29 | yes |
| `ffi-ptr-mem-chain:200` | 1,982,846 | 374,858 | 5.29 | yes |
| `ffi-strcpy-chain:200` | 4,102,245 | 405,087 | 10.13 | yes |
| `bfile-read-chain:200` | 1,398,217 | 275,511 | 5.07 | yes |
| `getenv-chain:200` | 1,469,514 | 344,612 | 4.26 | yes |
| `env-set-chain:200` | 1,958,847 | 602,186 | 3.25 | yes |
| `getcwd-chain:200` | 3,972,309 | 1,174,708 | 3.38 | yes |
| `file-read-close-chain:200` | 4,276,220 | 1,284,254 | 3.33 | yes |
| `utf8-bfile-read-chain:200` | 1,638,707 | 333,696 | 4.91 | yes |
| `crlf-bfile-read-chain:200` | 1,735,816 | 376,004 | 4.62 | yes |
| `buf-bfile-read-chain:200` | 1,676,470 | 349,403 | 4.80 | yes |
| `mvar-chain:200` | 20,784 | 117,743 | 0.18 | yes |
| `ptr-chain:200` | 80,302 | 107,577 | 0.75 | yes |
| `rnf-chain:200` | 75,782 | 3,801,194 | 0.02 | yes |
| `stableptr-chain:200` | 11,719 | 118,789 | 0.10 | yes |
| `weak-chain:200` | 20,320 | 115,161 | 0.18 | yes |
| `zoo-chain:300` | 151,606 | 122,531 | 1.24 | yes |
| `data-chain:300` | 93,757 | 131,540 | 0.71 | yes |

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

### Comment

| topic | current theory |
|---|---|
| Rust/C performance gap | The ignored-action shortcut confirms the gap is mostly semantic overhead, not parity noise: simple `IO.>>`/direct-FFI chains are now faster than C when the runtime can execute the ignored left action as a whole pattern. The remaining common gaps are concentrated in `IO.lazyBind` and guest-memory/BFILE paths, where the result of one action feeds the next and the Rust runtime still allocates pair/app nodes, walks reducer spines, and copies through adapters. |

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
| high-level environment tests | runtime now covers `getenv`, `setenv`, `unsetenv`, `environ`; benchmark covers mutation plus lookup | pending full high-level `System.Environment` test once the compiler binary is available |
| high-level errno/error tests | runtime now covers errno constants, `&errno`, `strerror_r`, and errno recording for implemented host failures | pending full high-level `Foreign.C.Error` / `throwErrnoIf*` tests once the compiler binary is available |
| high-level temp/CPU tests | runtime now covers `tmpname` and `getcpu`; direct smokes cover the raw FFI actions only | pending full high-level `System.IO.openTmpFile` / `System.CPUTime` tests once the compiler binary is available |
| commit runtime parity checkpoint | `63702f10 Add Rust mpz and JS FFI coverage` | done |
| JS full parity | runtimeFFI missing-symbol coverage is zero; `JSVal` object result/argument handles, wrapper creation boundary, owning-program handle plumbing, wrapper tag registry, internal StablePtr callback trampoline, typed JS wrapper callback API, real Rust `.wasm` build artifact, wasm embedding exports, browser-loadable JS host shim, direct wasm dynamic JS FFI smoke, same-program wrapper callback smoke, and broad wrapper tag coverage are in place, but high-level `foreign import javascript` browser parity is not complete; `.combffi` probing did not expose enough JavaScript import metadata for a runtime-only bridge | pending compiler-emitted metadata or generated glue path |
| performance phase | eight performance checkpoints plus one benchmark parity checkpoint committed; numeric benchmark matrix rows now all sink-match C; rows are split into common-case and rare/specialized; common-operation targets should continue with `IO.lazyBind`, guest-memory, and BFILE rows where Rust is still roughly 3x-10x slower than C | active |
| rejected performance probes | `KnownFfi` enum/symbol specialization, single-pass BFILE read dispatch, direct unit-return FFI pairing, direct Unix `getcwd` into guest allocation, head-node clone collapse in `step`, primitive-node cache, broad fixed-primitive borrowed classifier, pre-clone `IO.>>` branch, broad borrowed C-string host paths, borrowed `md5String`, over-broad pointer FFI pre-dispatch, generic-arm `peekPtr`/`pokePtr` direct returns, zero-arity FFI candidate guard, direct zero-arity FFI under `IO.>>`, direct FFI shortcut under `IO.lazyBind`, borrowed fixed-size peeks, runtime handle-table free lists, shared `Rc<str>` node symbols, and the `IO.return`/`K` continuation collapse were measured and reverted because important end-to-end rows were neutral or slower | done, do not reapply blindly |
| keep compiler Haskell | scope is C runtime/evaluator rewrite; Haskell compiler remains authoritative `.comb` producer | ongoing |
