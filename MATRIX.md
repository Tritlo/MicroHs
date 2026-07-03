# MicroHs Rust Matrix

Updated: 2026-07-03

This file is a tracked working status artifact. Update it when parity,
performance, or benchmark classification changes.

## Worktree State

| item | state |
|---|---|
| branch | `microhs-rust` |
| upstream tracking | `origin/microhs-rust` |
| local commits ahead after this snapshot commit | 97 |
| runtime code baseline | F2 Int/Int64 continuation-stack checkpoint |
| dirty files after this snapshot commit | none expected |
| dirty work | none in tracked runtime files |
| matrix file | `MATRIX.md`, tracked from this snapshot |

## Verification Baseline

Last fully verified state: F2 Int/Int64 continuation-stack checkpoint.

| gate | status |
|---|---|
| `cargo test -p microhs-runtime --quiet` | passed before F2 Int/Int64 continuation-stack commit; 34 tests |
| `cargo check -p microhs-runtime --lib --quiet` | passed before F2 Int/Int64 continuation-stack commit |
| `cargo check -p microhs-runtime --bins --quiet` | passed before F2 Int/Int64 continuation-stack commit |
| `cargo check --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed before F2 Int/Int64 continuation-stack commit |
| `cargo build --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed before F2 Int/Int64 continuation-stack commit |
| `node --check rust/microhs-runtime/js/host.mjs` | passed at `f8b9e1d5` |
| Node wasm core render smoke | passed at `9cbef47e`; host shim instantiated wasm, reduced `v8.4\n0\nI #5 @ }\n`, and rendered `5` |
| Node wasm dynamic `~I` JS FFI smoke | passed at `58280b6e`; path-loaded wasm reduced `IO.performIO ~I "return 40 + 2" @` and rendered `42` |
| Node wasm dynamic JS tag smoke | passed at `9cbef47e`; direct `~B`, `~S`, and `~J` smokes rendered `A`, `"hi"`, and `ForeignPtr#N` |
| Node wasm wrapper callback smoke | passed at `58280b6e`; byte-loaded wasm reduced `IO.performIO (IO.>>= (`II IO.return) (~IJ "return $0(35)"))` and rendered `35` |
| Node wasm wrapper tag coverage smoke | passed at `f8b9e1d5`; `II`, `UU`, `DD`, `FF`, `BB`, `SS`, `JJ`, and `PP` wrappers round-trip through JS and render expected values |
| Node wasm unsigned/high-bit JS smoke | passed at `f8b9e1d5`; direct `~U` rendered `4294967295`; direct and wrapper `~P` rendered `Ptr#2147483648` |
| Node wasm Response-source smoke | passed at `58280b6e`; `Response(bytes)` source reduced `~S "return 'hi'"` and rendered `"hi"` |
| `cargo fmt --all --check` | passed before F2 Int/Int64 continuation-stack commit |
| `git diff --check` | passed before F2 Int/Int64 continuation-stack commit |
| `make bin/mhsbench` | passed/up to date at `70eabf4d` |
| `cargo build --release -p microhs-runtime --bins --quiet` | passed before F2 Int/Int64 continuation-stack commit |
| signed `i64::MIN` comb parser smoke | passed before checkpoint commit; Rust now parses the self-host compiler comb containing `##-9223372036854775808` |
| unbounded internal force budget smoke | passed before checkpoint commit; self-hosting no longer trips the old internal `10_000` WHNF cap |
| main-mode benchmark harness smoke | passed before checkpoint commit; Rust and C support `--mode main -- PROGRAM ARGS...`; main mode measures execution and validates external output instead of serializing the whole root graph |
| self-host compiler input generation | passed before checkpoint commit; `./bin/mhs -i -imhs -isrc -ilib MicroHs.Main -o/tmp/mhs-selfhost.comb` produced a 647 KiB `v8.4` comb |
| self-host C main smoke | passed before checkpoint commit; updated `bin/mhsbench --mode main` produced `/tmp/mhs-selfhost-c-mainmode.comb`, a 647 KiB `v8.4` comb that parses under C WHNF smoke |
| lazy `readFile` CPP scan smoke | passed before checkpoint commit; temporary `hasLangCPP` reproducer now prints `False` under Rust like C, dropping the Rust one-shot time from ~433 ms before the fix to ~10 ms |
| self-host Rust main smoke | not yet passing; no-shim run no longer falsely invokes `cpphs`; latest full compile run before structural rewrites hit `timeout 900s` with exit `124`, printed only the missing `mhs.conf` warning to `/tmp/mhs-selfhost-rust-900s.out`, left empty stderr, and produced no output comb at `/tmp/mhs-selfhost-rust-900s.comb` |
| `performio-apply-chain:200` benchmark | passed before small-int cache commit; Rust/C sinks match at `130000`, Rust `77,333` ns/iter vs C `117,888` ns/iter |
| direct Scott-pair selector perf probe | passed before checkpoint commit; current probe recognizes `U K (P x y)`/`U A (P x y)` after forcing the pair to WHNF; `hasLangCPP` proxy drops from `23,392` to `23,226` reductions/iter and measured `4,576,282` ns/iter Rust vs `635,079` ns/iter C, while self-host still exceeds the 1,200s cutoff |
| UTF-8 ASCII refill perf probe | passed before checkpoint commit; text `readFile` proxy improved to `3,134,640` ns/iter, `hasLangCPP` proxy to `4,395,391` ns/iter, and self-host `--help` main smoke to `123,325,990` ns/iter Rust vs `12,412,585` ns/iter C |
| direct `IO.return` bind perf probe | passed before checkpoint commit; self-host `--help` main smoke improved to `101,524,666` ns/iter Rust vs `10,021,074` ns/iter C; text proxies and common canaries stayed in-band |
| dynamic runtime profiler probe | passed before checkpoint commit; `mhs-rust-bench --profile --profile-top N` runs one extra profiled iteration after normal timing and reports head attempts/reductions, spine arity, heap-spine count, max arity, final node count, and sink |
| standalone runtime profiler smoke | passed before checkpoint commit; `target/release/mhs-rust --profile --profile-top 5 /tmp/mhs-rust-profile-smoke.comb` prints normal WHNF output on stdout and profile counters on stderr |
| allocation profile counters probe | passed before checkpoint commit; profile output now reports node growth, app allocations, small-int cache hits/misses, and non-small Int allocations; self-host `--help` one-shot shows `profile_node_growth: 350676`, `profile_app_allocations: 347838`, `profile_small_int_cache_hits: 5660`, `profile_small_int_cache_misses: 2`, and `profile_non_small_int_allocations: 11` |
| self-host `--help` dynamic profile | passed before checkpoint commit; top reduction heads are `B` 62,065, `C` 36,472, `C'` 25,246, `C'B` 23,452, `P` 20,309; 120,563 heap spines and max arity 75 confirm the current bottleneck is reducer/spine throughput |
| uninitialized 16-slot inline spine perf probe | passed before checkpoint commit; self-host `--help` profile keeps the same 312,558 reductions and drops heap spines from 120,563 to 1,453; `MaybeUninit` avoids initializing unused inline slots, while `arith`/`zoo`/`data` canaries sink-match C under noisy timings |
| native Rust sampling profiler setup | installed `samply 0.13.1`; recording is currently kernel-blocked because `/proc/sys/kernel/perf_event_paranoid` is `2`; needs temporary `sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'` before `samply record --save-only ...` can collect samples |
| Rust/LLVM instrumentation profile | passed before profiling matrix commit; `llvm-tools-preview` installed, release bench rebuilt in `/tmp/mhs-rust-coverage-target` with `-C instrument-coverage -C force-frame-pointers=yes`, and self-host `--help` produced `/tmp/mhs-selfhost-help.profdata` plus `/tmp/mhs-selfhost-help-runtime-coverage.txt` |
| hot combinator app-reuse perf probe | passed before checkpoint commit; `B`, `C`, `C'`, `C'B`, and `P` now reuse the consumed redex app node for the final result app; self-host `--help` drops from `123,441,471` to `113,332,505` ns/iter and from `889,877` to `722,333` nodes after run with the same 312,558 reductions; rerun canaries sink-match C, with `arith`, `zoo`, and `data` better than prior matrix rows and `io`, `ffi-mem`, and `bfile-read` in the same noisy band |
| generalized combinator app-reuse perf probe | passed before checkpoint commit; final-app reuse now also covers pure combinator skeletons `U`, `S`, `S'`, `B'`, `Z`, `J`, `L`, `R`, `O`, partial `K2`/`K3`/`K4`, partial `C'B`, `Y`, `TAGn`, and `Tn`; self-host `--help` drops from `113,332,505` to `104,514,392` ns/iter and from `722,333` to `648,345` nodes after run with the same 312,558 reductions; rerun canaries sink-match C, with `arith`, `zoo`, and `data` better than prior matrix rows and `io`, `ffi-mem`, and `bfile-read` still in the same noisy band |
| `app_step!` reducer macro cleanup | passed before checkpoint commit; pure app-reuse reducer returns now use one local macro in `Program::step`; no intended behavior change; self-host `--help` canary still has sink `960903` and 312,558 WHNF steps, while raw timing was noisy on this machine |
| tuple first-field selector perf probe | passed before checkpoint commit; `Tn` now recognizes direct first-field selectors `K2`/`K3`/`K4` when enough tuple/extra args are already present, preserving the two-reduction count while skipping intermediate selector app allocation; `data-chain:300` improved to `138,919` ns/iter Rust vs `140,502` ns/iter C on the longer rerun; self-host `--help` kept sink `960903`, 312,558 reductions, and 648,345 nodes after run, so this is a data-constructor win rather than a compiler-scale win |
| resolve/shortcut profiler probe | passed before checkpoint commit; runtime profile now reports resolve-call count, total/max indirection hops, resolve-depth histogram, and shortcut-hit counts; self-host `--help` has 5,566,726 profiled resolve calls but only 99,382 followed indirections and max chain 4, so repeated node classification/spine traversal is a larger signal than deep indirection chains |
| tagged primitive representation perf probe | passed before checkpoint commit; parsed/created primitives now store `KnownPrim` tags for the hot combinator/IO/tag/tuple set and preserve names for render/serialize; canaries sink-match C; `zoo-chain:300` and `data-chain:300` drop to ~64 us/iter in this run, and self-host `--help` drops to `78,222,728` ns/iter with the same 312,558 reductions and 648,345 profile nodes |
| direct known primitive dispatch perf probe | passed before checkpoint commit; `Program::step` now dispatches hot combinator/IO/tag/tuple primitives on `KnownPrim` instead of string names; sequential canaries sink-match C, with `arith`, `zoo`, `data`, `io`, and `ffi` faster than C in this run and `bfile-read` still behind; self-host `--help` improved to `72,272,334` ns/iter with the same 312,558 reductions and 648,345 profile nodes |
| no-profile resolve fast-path perf probe | passed before checkpoint commit; normal runs now bypass the dynamic resolve-profile hook instead of making a cold no-op call for each resolve, while `--profile` keeps the same resolve-depth histogram; sequential canaries sink-match C, with `arith`, `zoo`, `data`, `io`, and `ffi` in-band or slightly better than the direct-dispatch checkpoint; same-session absolute-path self-host `--help` comparison was `75,119,651` ns/iter vs `76,337,245` for `b0685b62`, and the usual local `./bin/mhs` proxy was `73,041,815` ns/iter with the same 312,558 reductions and 648,345 profile nodes |
| known-primitive shortcut-helper perf probe | passed before checkpoint commit; hot IO/pair/selector helper predicates now compare `KnownPrim` tags instead of primitive strings; sequential canaries sink-match C, `bfile-read-chain:200` improved to `288,388` ns/iter in this noisy run, and self-host `--help` stayed count-identical at `73,030,710` ns/iter, 312,558 reductions, and 648,345 profile nodes |
| C-style `Y` knot perf probe | passed before checkpoint commit; `T_Y` now reuses the consumed `(Y x)` redex as the recursive argument like C's `GOAP(x, n)`, and no-op self-updates no longer write self-indirections; `Y K` reaches WHNF as a cycle, `Y I` hits the step limit rather than hanging, common canaries sink-match C, and self-host `--help` drops to 297,417 reductions and 624,314 profile nodes after run |
| C-compatible quoted bytestring serializer smoke | passed before checkpoint commit; quoted serialization now follows C/`ExpPrint.hs` for escape introducers, leaves literal `?` raw, uses `\?` only for DEL, and round-trips every byte through the quoted parser path |
| C-compatible Integer serializer smoke | passed before checkpoint commit; `%` parser now accepts C's `%digits"` wire format while tolerating legacy Rust `%"digits"`, and serializer emits C-style `%digits"` for `BigInt`/mpz foreign pointers |
| bounded ignored-IO shortcut smoke | passed before checkpoint commit; ignored-action preflight/execution now caps native recursion at 256 nested shortcut actions and falls back to the general reducer beyond that; `io-chain:200` and `io-control-chain:200` still sink-match C |
| eval.c FFI table completion smoke | passed before checkpoint commit; `putchar` arity 1 and `lz77c` arity 3 are now in Rust's FFI arity table and call dispatcher; direct smokes cover `putchar` through `IO.performIO` and `lz77c` pointer-to-pointer compression with Rust decompression |
| C-compatible `fromUTF8` malformed-input smoke | passed before checkpoint commit; `fromUTF8` now follows C's BFILE UTF-8 decoder for malformed inputs: accepts modified-UTF8 NUL, rejects overlong nonzero encodings, and treats a truncated tail as EOF; C's separate `headUTF8`/`tailUTF8` helper remains unchanged |
| catchable arithmetic RTS exception smoke | passed before checkpoint commit; Int/Int64 divide-by-zero and signed-overflow primitive origins now raise catchable RTS exception nodes 4/7 under `catch`, while invalid shift and internal runtime overflow remain terminal runtime errors |
| unknown primitive hard-failure smoke | passed before checkpoint commit; truly unknown bare primitive names now fail during `.comb` parse, C-known but currently unsupported primitives such as `IO.fork`/`IO.deserialize` still load but fail loudly if reduced with arguments, and `/tmp/mhs-selfhost.comb` parses/runs the `--help` main proxy under this gate |
| C-compatible uncaught exception display smoke | passed before checkpoint commit; raw RTS exception ints use C's `die_exn` message table, non-RTS exceptions are displayed by evaluating `U (U (K2 A)) exn`, and a compiled `exitSuccess` comb now completes main-mode release benchmark execution instead of panicking |
| compression cross-runtime fixture smoke | passed before checkpoint commit; Rust unit fixtures decode C-runtime LZ77/BWT/LZMA frames for an adversarial repeated/high-byte/NUL payload, Rust compressor/decompressor self-roundtrips the same payload, and an external C-runtime smoke decoded Rust-produced LZ77/BWT/LZMA frames as `(True,True,True)` |
| buffered stdio write smoke | passed before checkpoint commit; stdout/stderr writes no longer flush after every byte/buffer write, while explicit flush/close paths still call `flush_io_handle`; no dedicated output-heavy timing row exists yet |
| cached world token perf probe | passed before checkpoint commit; `performio-apply-chain:200` improved to Rust `78,559` ns/iter vs C `124,271` with sink `130000`, `io-chain:200` stayed in-band at Rust `63,910` vs C `121,829` with sink `132000`, and self-host `--help` proxy ran at `69,636,341` ns/iter with unchanged 297,417 steps |
| small-int cache perf probe | passed before checkpoint commit; runtime-created Int nodes in C's `-10..255` table range now reuse parsed/cached nodes, common canaries sink-match C (`arith-chain:200` Rust `59,394` vs C `105,195`, `bytes-chain:200` Rust `97,070` vs C `163,531`, `io-chain:200` Rust `65,292` vs C `127,276`, `performio-apply-chain:200` Rust `77,333` vs C `117,888`), and self-host `--help` profile nodes drop to `615,854` with unchanged 297,417 steps; raw self-host timing was mixed/slower (`73,192,042` ns/iter over 3 iters, `74,562,989` over 5), so this is a graph-size/C-shape checkpoint rather than a claimed speed win |
| strict WHNF coercion fast-path perf probe | passed before checkpoint commit; strict coercion helpers now return immediately when their argument already resolves to the expected WHNF tag, matching eval.c's fast operand checks before the full marker-continuation rewrite; canaries sink-match C (`arith-chain:200` Rust `60,494` vs C `111,854`, `bytes-chain:200` Rust `87,891` vs C `170,867`, `io-chain:200` Rust `69,336` vs C `130,358`), and self-host `--help` improves to `67,006,399` ns/iter with unchanged 297,417 steps and sink `913271`; profile step attempts drop from `322,900` to `305,865` and resolve calls from `5,334,783` to `5,317,748`, with unchanged `350,676` node growth |
| F2 Int/Int64 continuation-stack probe | passed before checkpoint commit; `eval_int` and non-shift `eval_int64` now force nested unary/binary primops through explicit inline-one-frame continuation stacks instead of recursively calling `reduce_node_whnf`; `arith-chain:200` is Rust `36,122` ns/iter vs C `108,853`, `int64-chain:200` improves to Rust `35,442` ns/iter vs C `112,619`, and self-host `--help` recovers from the Int-only `80,617,930` ns/iter to `74,482,344`; still a structural checkpoint rather than a compiler-speed win |
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
| runtimeFFI coverage script | passed at `63702f10` for the compiler `runtimeFFI` list; later `eval.c` table review found the list missed `putchar` and `lz77c`, both now implemented directly |
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

Latest tracked runtime behavior change is the first F2 marker-continuation slice: `eval_int` and non-shift `eval_int64` handle nested unary/binary primops with explicit continuation stacks instead of native recursive reducer calls.

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
| `arith-chain:200` | 36,122 | 108,853 | 0.33 | yes |
| `int64-chain:200` | 35,442 | 112,619 | 0.31 | yes |
| `float64-chain:200` | 61,633 | 112,001 | 0.55 | yes |
| `float32-chain:200` | 63,676 | 116,081 | 0.55 | yes |
| `bytes-chain:200` | 87,891 | 170,867 | 0.51 | yes |
| `cstring-pack:200` | 1,706 | 102,501 | 0.02 | yes |
| `foreignptr-slice:200` | 1,494 | 94,820 | 0.02 | yes |
| `unpack-chain:200` | 8,678 | 97,157 | 0.09 | yes |
| `fromutf8-chain:200` | 9,939 | 97,162 | 0.10 | yes |
| `array-chain:200` | 3,572 | 91,485 | 0.04 | yes |
| `io-chain:200` | 69,336 | 130,358 | 0.53 | yes |
| `io-array-chain:200` | 3,747 | 92,561 | 0.04 | yes |
| `io-bytes-chain:200` | 1,814 | 90,873 | 0.02 | yes |
| `io-control-chain:200` | 59,303 | 131,295 | 0.45 | yes |
| `performio-apply-chain:200` | 77,333 | 117,888 | 0.66 | yes |
| `argref-chain:200` | 55,481 | 128,597 | 0.43 | yes |
| `stdio-chain:200` | 143,397 | 112,014 | 1.28 | yes |
| `ffi-chain:200` | 60,830 | 152,403 | 0.40 | yes |
| `ffi-math-chain:200` | 82,002 | 170,476 | 0.48 | yes |
| `ffi-const-chain:200` | 76,276 | 200,473 | 0.38 | yes |
| `ffi-mem-chain:200` | 183,052 | 233,483 | 0.78 | yes |
| `ffi-wide-mem-chain:200` | 4,591,002 | 412,413 | 11.13 | yes |
| `ffi-word-mem-chain:200` | 3,996,918 | 373,408 | 10.70 | yes |
| `ffi-ptr-mem-chain:200` | 1,975,729 | 359,302 | 5.50 | yes |
| `ffi-strcpy-chain:200` | 4,358,649 | 401,217 | 10.86 | yes |
| `bfile-read-chain:200` | 295,939 | 274,019 | 1.08 | yes |
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
| `zoo-chain:300` | 60,275 | 123,000 | 0.49 | yes |
| `data-chain:300` | 57,396 | 136,987 | 0.42 | yes |

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
| `--help` main smoke | 9,934,460 ns/iter; sink `661902` per iter | 74,482,344 ns/iter; sink `913271` per iter | both run and print identical usage text; not a full compile; refreshed with `--warmup-iters 1 --iters 3` after the F2 Int/Int64 continuation-stack slice; this fixes part of the strict-chain shape and speeds arithmetic/int64, but remains slower than the strict-WHNF proxy checkpoint pending broader marker-machine work |
| self-host compiler smoke | 56,387,928,187 ns/iter; output `/tmp/mhs-selfhost-c-refresh.comb` is a 647 KiB `v8.4` comb | latest full run before structural rewrites hit `timeout 900s` with exit `124`; only `/tmp/mhs-selfhost-rust-900s.out` warning was produced, stderr was empty, and no output comb existed at `/tmp/mhs-selfhost-rust-900s.comb` | not at parity |

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

`timeout 60s target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main --warmup-iters 0 --iters 1 --profile --profile-top 20 -- ./bin/mhs --help > /tmp/mhs-selfhost-y-knot-profile20.out`

In the bench harness, `--profile` runs one extra profiled iteration after normal
timing, so the normal benchmark output stays comparable. The standalone
`mhs-rust --profile --profile-top N` path prints the same counters to stderr
while leaving the rendered result on stdout.

| measure | count | implication |
|---|---:|---|
| profiled iteration time | 123.921 ms | instrumentation overhead is separate from normal timing; raw time is noisy |
| reductions | 297,417 | C-style `Y` knot removes 15,141 self-host `--help` reductions versus the prior checkpoint |
| step attempts | 322,900 | only 28,346 attempts do not reduce; most work is real reduction |
| successful step heads | 294,554 | optimized multi-reduction steps account for the gap to total reductions |
| nodes after run | 624,314 | C-style `Y` knot removes 24,031 transient nodes from this proxy versus the prior checkpoint |
| heap spines | 1,302 | increasing inline storage to 16 still covers almost all hot arities without heap allocation |
| max spine arity | 75 | long application spines are present even in the short `--help` path |
| profiled resolve calls | 5,334,783 | resolve/node-classification traffic is much larger than reduction count, but the `Y` knot removes about 232k calls from this proxy |
| followed indirections | 99,346 | only about 1.9% of profiled resolve calls follow an indirection |
| max resolve chain | 4 | deep indirection chains are not the current self-host `--help` bottleneck |
| shortcut hits | `selector_pair_field` 24; `identity_alias_chain` 5 | existing direct shortcuts barely fire in this proxy |

Top dynamic reduction heads:

| rank | head | reductions |
|---:|---|---:|
| 1 | `Prim:B` | 56,409 |
| 2 | `Prim:C` | 36,472 |
| 3 | `Prim:C'` | 25,246 |
| 4 | `Prim:P` | 20,309 |
| 5 | `Prim:C'B` | 20,142 |
| 6 | `Prim:K` | 14,202 |
| 7 | `Prim:S` | 13,980 |
| 8 | `Prim:Z` | 13,932 |
| 9 | `Prim:O` | 8,986 |
| 10 | `Prim:A` | 8,863 |
| 11 | `Prim:S'` | 8,476 |
| 12 | `Prim:IO.>>=` | 8,397 |
| 13 | `Prim:IO.>>` | 5,716 |
| 14 | `Prim:IO.return` | 5,602 |
| 15 | `Prim:B'` | 5,572 |
| 16 | `Prim:T3` | 5,569 |
| 17 | `Prim:R` | 5,562 |
| 18 | `Prim:K3` | 5,560 |
| 19 | `Prim:U` | 2,973 |
| 20 | `Prim:==` | 2,869 |

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

Resolve depth distribution:

| depth | calls |
|---:|---:|
| 0 | 5,475,758 |
| 1 | 82,560 |
| 2 | 8,404 |
| 3 | 2 |
| 4 | 2 |

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
| primitive dispatch | hot fall-through through the former `Prim` string match cascade | direct `KnownPrim` dispatch is now in place; remaining hot cost is the evaluator shape around spine walk, node classification, and app update/rebuild |

### Lessons From `eval.c`

This is the current read of Lennart's C evaluator, mapped to the Rust runtime
work. `implemented` tracks whether the Rust runtime has the same broad
performance idea, not merely whether it has matching behavior.

| lesson | `eval.c` signal | Rust implication | implemented |
|---|---|---|---|
| Use a compact node representation | Low tag bits distinguish `T_AP`, `T_IND`, and tagged payload nodes inside one heap cell | Rust's `Node` enum is simpler and safer, but it likely costs more branch/data movement in the hot path; do not attempt this before cheaper evaluator-loop work | no |
| Represent primitive heads as tags, not strings | `enum node_tag`, permanent primitive nodes, and the `primops[]` table feed a tag switch in `evali` | Keep `.comb` names at boundaries, but push more hot reducer and helper dispatch through `KnownPrim`; extend beyond combinators/IO only if profiles justify it | partial |
| Walk application spines directly on the evaluator stack | `evali` follows `T_AP` nodes with `PUSH(n)` until the head tag is known | Rust's owned `Spine` object, argument reversal, and repeated resolve/node classification are now the main structural gap; a stack-like reducer loop is the highest-value C-shaped experiment | no |
| Rewrite the consumed redex cell and continue | `GOIND`, `GOAP`, and `GOAP2` mutate the current node/root app and jump back to `top`/`ap` | App-reuse work moved in this direction, but Rust still returns `StepResult` and rebuilds extra arguments through helper slices instead of staying in one tight evaluator loop | partial |
| Tie fixpoint knots in the consumed redex | `T_Y` does `GOAP(x, n)`, so `n@(Y x)` becomes `x n` instead of allocating a fresh `(Y x)` | Rust now reuses `apps[0]` as the recursive argument for `Y`, which preserves sharing and reduces self-host `--help` reductions/nodes; no-op self-updates avoid self-indirection hangs for divergent cycles like `Y I` | yes |
| Treat common values as permanent/cached nodes | `init_nodes` creates permanent primitive nodes, small ints live in `intTable`, and helpers reuse `combK`, `combB`, `combIOBIND`, `combWorld`, etc. | Rust has `PrimCache` for a small set of heads, reuses the world token, and caches runtime-created small Ints seeded from parsed literals; it still lacks permanent parsed primitive singletons, and the small-int cache trimmed self-host graph size without a raw speed win | partial |
| Use marker continuations for strict primitive forcing | Binary/unary int, int64, float, double, and bytes primops push nodes such as `T_BININT2`/`T_BININT1` and finish in `ret`, with fast paths when operands are already boxed values | Rust now has cheap fast paths for already-WHNF coercion arguments plus explicit continuation stacks for nested Int and non-shift Int64 unary/binary forcing; mixed Int64 shifts, float/bytes, and the main reducer loop still need the same treatment | partial |
| Hand-shape arity-specific rewrites | `T_T3`..`T_T16`, `T_TAG0`..`T_TAG32`, `B`, `C`, `C'B`, `P`, and partial `K2`/`K3`/`K4` have direct switch arms | Rust has direct `KnownPrim` arms, reducer macros, tuple first-field shortcuts, and app reuse, but generic spine rebuild still costs more than C's macro-shaped rewrites | partial |
| Simplify graph fragments when traversal is already happening | `GCRED` folds `A/K/I`, `B I`, `B x I`, `C op`, `C' I`, and related shapes while marking | Rust has no parse/GC simplification pass; current profiles point to reducer shape first, so this is a remembered later option rather than an active target | no |
| Encode ordinary IO combinators as combinator rewrites | `T_IO_BIND` jumps to `T_C`, `T_IO_RETURN` jumps to `T_P`, and `T_IO_THEN` builds bind plus `K` | Rust mirrors the behavior and adds profiled shortcuts for ignored actions, direct returned binds, and lazyBind/FFI cases; more IO work needs profile evidence | partial |
| Keep counters close to the evaluator loop | `-v`, `WANT_KPERF`, `GCRED`, reduction counters, allocation counters, and selected special-reduction counters live beside evaluator code | Rust now has benchmark/profile output, resolve-depth counters, shortcut counters, app-allocation counters, small-int cache counters, node-growth reporting, and LLVM coverage/hot counts; normal runs bypass the profiling hook, but native sampling is still blocked by kernel perf settings | partial |

### Comment

| topic | current theory |
|---|---|
| Rust/C performance gap | The ignored-action, direct lazyBind, performIO extra-spine, direct Scott-pair selector, UTF-8 ASCII refill, direct `IO.return` bind, uninitialized 16-slot inline spine, hot/generalized combinator app-reuse probes, tuple first-field selector shortcut, dynamic runtime counters, resolve-depth counters, tagged primitive representation, direct `KnownPrim` dispatch, no-profile resolve fast path, known-primitive shortcut-helper cleanup, C-style `Y` knot, cached world token, small-int cache, allocation counters, strict WHNF coercion fast paths, and Rust/LLVM hot-count profile confirm the gap is mostly evaluator overhead, not parity noise. Simple `IO.>>`, direct FFI, unary result-fed FFI/BFILE chains, lazy performIO application, ASCII text reads, trivial returned binds, constructor first-field selectors, tagged primitive clone avoidance, direct known-head/helper dispatch, fixpoint sharing, and most self-host `--help` spine traversals are now close to C or materially better than before. The static, dynamic, LLVM, and `eval.c` reads point away from IO/array/FFI as the main full-compiler blocker. Because this machine may be busy, raw timings are treated as indicative only; the stronger signal is the count distribution: small-int caching lowered self-host profile nodes to 615,854 with unchanged reductions, but did not improve the raw self-host proxy; strict WHNF coercion fast paths then cut profile step attempts by 17,035 and resolve calls by 17,035 without changing node growth. The active theory is compile-scale reducer/runtime throughput: Rust now avoids known-primitive string dispatch, normal-run resolve profiling overhead, fresh `Y` unrolling, some duplicate common nodes, and some already-WHNF coercion reducer entries, but it still pays for owned spine construction/reversal, repeated resolve/node-classification, helper-return dispatch, extra-argument rebuild, and transient app allocation where C stays in one stack/goto evaluator loop. Start-node resolve-chain compression and saturated-redex root updates looked C-shaped but regressed the self-host proxy, and small-int caching shows graph-size wins alone are not enough, so the next work should be a more structural reducer-loop/spine representation change. |

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
| detailed `eval.c` correctness batch | finalized review ranks small oracle-checkable semantic fixes before the structural reducer rewrite | F1 quoted bytestring serializer, F3 catchable arithmetic RTS exceptions, F4 uncaught exception display/`ExitSuccess`, F7 unknown primitive hard-failure, F11 Integer serializer, F15 missing FFI symbols, F16 `fromUTF8` malformed input, F18 ignored-IO shortcut cap, and F19 compression cross-runtime fixtures done |
| compression BFILE write-path benchmarks | decompressor rows existed; compiler-generated high-level compressor smokes now cover RLE/LZ77/BWT/LZMA write paths | partial done; built-in repeat scenarios still pending |
| MD5 broader coverage | committed runtime covers all three FFI names; only `md5String` has a repeat benchmark | pending full high-level `System.IO.MD5` test once the compiler binary is available |
| directory iteration FFI | committed runtime covers `opendir`, `readdir`, `closedir`, `c_d_name` | pending full high-level `System.Directory` test once the compiler binary is available |
| self-hosting parity | C runtime can execute the compiler comb and produce a new compiler comb | Rust main-mode harness exists and lazy file IO now matches the C CPP scan smoke; latest full-compile run still hit the 900s timeout with no output comb; inline spine work removed almost all `--help` heap-spine spills, generalized combinator app reuse cuts self-host `--help` node growth by ~27%, direct known-head/helper dispatch plus C-style `Y` sharing keep the short self-host proxy in the ~70-75 ms band, small-int caching trims final profile nodes to 615,854 without speeding the proxy, strict WHNF coercion fast paths improved the proxy to ~66 ms, and the first F2 Int/Int64 continuation-stack slices improve arithmetic/int64 but leave this proxy at ~74 ms; full self-host parity still needs the broader structural reducer/marker machine |
| high-level environment tests | runtime now covers `getenv`, `setenv`, `unsetenv`, `environ`; benchmark covers mutation plus lookup | pending full high-level `System.Environment` test once the compiler binary is available |
| high-level errno/error tests | runtime now covers errno constants, `&errno`, `strerror_r`, and errno recording for implemented host failures | pending full high-level `Foreign.C.Error` / `throwErrnoIf*` tests once the compiler binary is available |
| high-level temp/CPU tests | runtime now covers `tmpname` and `getcpu`; direct smokes cover the raw FFI actions only | pending full high-level `System.IO.openTmpFile` / `System.CPUTime` tests once the compiler binary is available |
| commit runtime parity checkpoint | `63702f10 Add Rust mpz and JS FFI coverage` | done |
| JS full parity | runtimeFFI missing-symbol coverage is zero; `JSVal` object result/argument handles, wrapper creation boundary, owning-program handle plumbing, wrapper tag registry, internal StablePtr callback trampoline, typed JS wrapper callback API, real Rust `.wasm` build artifact, wasm embedding exports, browser-loadable JS host shim, direct wasm dynamic JS FFI smoke, same-program wrapper callback smoke, and broad wrapper tag coverage are in place, but high-level `foreign import javascript` browser parity is not complete; `.combffi` probing did not expose enough JavaScript import metadata for a runtime-only bridge | pending compiler-emitted metadata or generated glue path |
| performance phase | structural F2 work active after twenty-three performance checkpoints plus one benchmark parity checkpoint plus static, dynamic, Rust/LLVM, resolve/shortcut/allocation profile, and `eval.c` lesson checkpoints; numeric benchmark matrix rows now all sink-match C; rows are split into common-case and rare/specialized; common-operation targets have shifted from narrow IO/FFI/data-constructor probes to C-shaped reducer work: explicit marker continuations, stack-like spine traversal, repeated resolve/node-classification traffic, helper-return dispatch, and app allocation/update mechanics | active |
| rejected performance probes | `KnownFfi` enum/symbol specialization, single-pass BFILE read dispatch, direct unit-return FFI pairing, direct Unix `getcwd` into guest allocation, head-node clone collapse in `step`, primitive-node cache, broad fixed-primitive borrowed classifier, pre-clone `IO.>>` branch, broad borrowed C-string host paths, borrowed `md5String`, over-broad pointer FFI pre-dispatch, generic-arm `peekPtr`/`pokePtr` direct returns, zero-arity FFI candidate guard, direct zero-arity FFI under `IO.>>`, direct FFI shortcut under `IO.lazyBind`, borrowed fixed-size peeks, runtime handle-table free lists, shared `Rc<str>` node symbols, `IO.>>=` direct array-action execution, the `IO.return`/`K` continuation collapse, broad fast-combinator pre-dispatch, skipping outer-app rethreading after reduction, always-inlining the hot small reducer helpers, 12-slot inline spine storage, 8-slot uninitialized-only spine storage, lazy argument resolution in `spine`, broad IO final-app reuse, start-node resolve-chain compression, non-App WHNF driver fast-exit, narrow `force_whnf` helper at strict call sites, `isInt`-only non-App force skip, lazy short-circuiting generic primitive cascade, generic final-spine `Int` redex update, `KnownPrim` tagging for `Int` ops, reverse-free inline spine layout, `StepResult.in_place` removal, and in-reducer saturated-redex root updates were measured and reverted or not selected because important end-to-end rows were neutral or slower | done, do not reapply blindly |
| keep compiler Haskell | scope is C runtime/evaluator rewrite; Haskell compiler remains authoritative `.comb` producer | ongoing |
