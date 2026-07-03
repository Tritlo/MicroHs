# MicroHs Rust Matrix

Updated: 2026-07-03

This is the current status matrix for the Rust rewrite of the MicroHs C
runtime/evaluator. The Haskell compiler stays in Haskell; this work is the
runtime/evaluator and host support for `.comb`.

## Current State

| item | state |
|---|---|
| branch | `microhs-rust` |
| upstream tracking | `origin/microhs-rust` |
| local commits ahead after this snapshot commit | 117 |
| runtime baseline | F2 generic strict coercion fold and rewrite/materialization counters |
| tracked dirty files expected after commit | none |
| untracked notes | `EVALLESSONS.md` is local working material |

## Verification

Last verified checkpoint: `01436e6f Fold generic strict coercions`.

| gate | status |
|---|---|
| `cargo fmt --all --check` | passed |
| `git diff --check` | passed before commit |
| `cargo check -p microhs-runtime --lib --quiet` | passed |
| `cargo check -p microhs-runtime --bins --quiet` | passed |
| `cargo test -p microhs-runtime --quiet` | passed, 34 tests |
| `cargo build --release -p microhs-runtime --bins --quiet` | passed |
| `cargo check --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed |
| `cargo build --target wasm32-unknown-unknown -p microhs-runtime --lib --quiet` | passed |
| `make bin/mhsbench` | up to date from earlier checkpoint |

## Runtime Coverage

| area | status | notes |
|---|---|---|
| `.comb` parser/serializer | comparable | version `v8.4`, labels, literals, application, C-compatible string/Integer output |
| core reducer/combinators | comparable | hot SK/zoo/data paths benchmarked against C |
| numeric primops | comparable | Int, Int64, Float64, Float32; strict forcing mostly owned by the WHNF loop |
| ByteString primops | comparable | core bytes/list/UTF-8 paths covered; ByteString benchmarks sink-match C |
| arrays and mutable bytes | comparable | benchmark uses the IO/world-shaped contract C implements |
| IO core/exceptions/masking stubs | comparable | catch/raise, RTS arithmetic exceptions, uncaught display, basic IO covered |
| MVar/StablePtr/Weak/ForeignPtr | comparable smoke | single-threaded semantics covered; ForeignPtr finalizers still wait for GC |
| FFI constants/math/memory | comparable | dynamic FFI subset, errno, malloc/calloc/realloc/free, typed peek/poke |
| filesystem/env/process FFI | comparable smoke | native behavior and errno recording implemented for covered calls |
| BFILE transducers | comparable smoke | memory/file/UTF-8/CRLF/buffer/RLE/base64/LZ77/BWT/LZMA paths covered |
| MD5/compression | comparable rare paths | present and sink-matching; not common-case performance targets |
| JS FFI hooks | partial | wasm host shim and wrapper smokes pass; high-level browser metadata/glue still pending |
| mpz/imath FFI | comparable smoke | decimal-backed `MpzValue`; correctness smokes pass, performance not a target yet |
| Haskell compiler | kept in Haskell | Rust consumes compiler-produced `.comb` |

## Benchmark Protocol

| setting | value |
|---|---|
| Rust command | `target/release/mhs-rust-bench --scenario <name> --iters 1000 --warmup-iters 100 --c-mhsbench ./bin/mhsbench` |
| C command | invoked by Rust harness via `./bin/mhsbench` |
| default C mode | `whnf` |
| comparison rule | sink must match before timing is meaningful |
| ratio | Rust ns/iter divided by C ns/iter |
| caveat | this machine may be busy; trust sinks, work counts, and profile distributions over one-off raw timings |

## Common Benchmarks

These rows are sink-comparable and remain the first performance target.

| scenario | Rust ns/iter | C ns/iter | ratio | sink |
|---|---:|---:|---:|---|
| `identity-chain:1000` | 37,794 | 140,241 | 0.27 | yes |
| `arith-chain:200` | 35,306 | 108,521 | 0.33 | yes |
| `int64-chain:200` | 37,806 | 108,862 | 0.35 | yes |
| `float64-chain:200` | 46,167 | 126,383 | 0.37 | yes |
| `float32-chain:200` | 45,533 | 127,072 | 0.36 | yes |
| `bytes-chain:200` | 56,110 | 181,975 | 0.31 | yes |
| `cstring-pack:200` | 1,706 | 102,501 | 0.02 | yes |
| `foreignptr-slice:200` | 1,494 | 94,820 | 0.02 | yes |
| `unpack-chain:200` | 8,678 | 97,157 | 0.09 | yes |
| `fromutf8-chain:200` | 9,939 | 97,162 | 0.10 | yes |
| `array-chain:200` | 3,572 | 91,485 | 0.04 | yes |
| `io-chain:200` | 69,336 | 130,358 | 0.53 | yes |
| `io-array-chain:200` | 3,747 | 92,561 | 0.04 | yes |
| `io-bytes-chain:200` | 1,814 | 90,873 | 0.02 | yes |
| `io-control-chain:200` | 53,668 | 142,576 | 0.38 | yes |
| `performio-apply-chain:200` | 65,891 | 125,436 | 0.53 | yes |
| `argref-chain:200` | 55,481 | 128,597 | 0.43 | yes |
| `stdio-chain:200` | 143,397 | 112,014 | 1.28 | yes |
| `ffi-chain:200` | 60,830 | 152,403 | 0.40 | yes |
| `ffi-math-chain:200` | 82,002 | 170,476 | 0.48 | yes |
| `ffi-const-chain:200` | 76,276 | 200,473 | 0.38 | yes |
| `ffi-mem-chain:200` | 183,052 | 233,483 | 0.78 | yes |
| `bfile-read-chain:200` | 295,939 | 274,019 | 1.08 | yes |
| `mvar-chain:200` | 20,784 | 117,743 | 0.18 | yes |
| `ptr-chain:200` | 80,302 | 107,577 | 0.75 | yes |
| `rnf-chain:200` | 85,682 | 4,673,986 | 0.02 | yes |
| `stableptr-chain:200` | 11,719 | 118,789 | 0.10 | yes |
| `weak-chain:200` | 20,320 | 115,161 | 0.18 | yes |
| `zoo-chain:300` | 54,581 | 134,304 | 0.41 | yes |
| `data-chain:300` | 59,828 | 141,740 | 0.42 | yes |

## Rare/Specialized Benchmarks

These rows are sink-comparable, but lower priority for F2 because they cover
codec, filesystem, errno, or host-heavy paths.

| scenario | Rust ns/iter | C ns/iter | ratio | sink |
|---|---:|---:|---:|---|
| `ffi-wide-mem-chain:200` | 4,591,002 | 412,413 | 11.13 | yes |
| `ffi-word-mem-chain:200` | 3,996,918 | 373,408 | 10.70 | yes |
| `ffi-ptr-mem-chain:200` | 1,975,729 | 359,302 | 5.50 | yes |
| `ffi-strcpy-chain:200` | 4,358,649 | 401,217 | 10.86 | yes |
| `getenv-chain:200` | 930,970 | 370,071 | 2.52 | yes |
| `env-set-chain:200` | 675,467 | 646,157 | 1.05 | yes |
| `getcwd-chain:200` | 3,972,309 | 1,174,708 | 3.38 | yes |
| `file-read-close-chain:200` | 3,681,595 | 1,334,167 | 2.76 | yes |
| `utf8-bfile-read-chain:200` | 402,575 | 341,293 | 1.18 | yes |
| `crlf-bfile-read-chain:200` | 424,645 | 324,672 | 1.31 | yes |
| `buf-bfile-read-chain:200` | 1,618,560 | 360,791 | 4.49 | yes |
| `md5-string-chain:200` | 4,378,292 | 455,086 | 9.62 | yes |
| `errno-chain:200` | 4,126,808 | 446,975 | 9.23 | yes |
| `dir-read-chain:200` | 9,372,765 | 3,237,676 | 2.89 | yes |
| `remove-missing-chain:200` | 642,622 | 368,672 | 1.74 | yes |
| `base64-bfile-read-chain:200` | 1,685,450 | 341,881 | 4.93 | yes |
| `lz77-bfile-read-chain:200` | 2,555,758 | 1,119,364 | 2.28 | yes |
| `bwt-bfile-read-chain:200` | 1,607,342 | 366,929 | 4.38 | yes |
| `lzma-bfile-read-chain:200` | 1,932,381 | 507,415 | 3.81 | yes |
| `rle-bfile-read-chain:200` | 1,692,821 | 348,352 | 4.86 | yes |

## Self-Hosting

Self-hosting means running the Haskell MicroHs compiler comb under the runtime
to compile `MicroHs.Main` back to a `.comb`. The compiler stays Haskell.

| check | C runtime | Rust runtime | status |
|---|---:|---:|---|
| compiler comb input | 647 KiB `/tmp/mhs-selfhost.comb` | parses | input ready |
| `--help` main proxy | 11,152,613 ns/iter; sink `661902` | 64,076,443 ns/iter; sink `661902` | runs, prints identical usage, about 5.7x C |
| full self-host compile | 61,920,977,046 ns/iter; emits 647 KiB comb | latest Rust run timed out at 900s with no output comb | not at parity |

## Self-Host Profile

Command:

```sh
target/release/mhs-rust-bench --input /tmp/mhs-selfhost.comb --mode main \
  --warmup-iters 0 --iters 1 --profile --profile-top 12 -- ./bin/mhs --help
```

| measure | count | implication |
|---|---:|---|
| profiled iteration time | 100.653 ms | instrumentation overhead; do not compare directly with normal timing |
| reductions | 297,417 | F2 changes are preserving work count |
| step attempts | 300,286 | most loop trips do real reduction |
| successful step heads | 294,554 | multi-reduction shortcuts explain the gap to total reductions |
| nodes after run | 615,854 | graph size unchanged by current F2 loop work |
| app allocations | 347,838 | allocation/rewrite traffic remains central |
| generic arg materializations | 8,523 / 37,356 nodes | measurable but secondary |
| spine rewrites | 62,496 / 251,284 extra args | result rewrites rebuild substantial tails |
| app rewrites | 226,403 / 1,300,478 extra args | current strongest signal for F2 performance work |
| heap spines | 1,302 | 16-slot inline spine covers almost all hot arities |
| max spine arity | 75 | long spines exist even in `--help` |
| profiled resolve calls | 5,014,674 | resolve/classification traffic is much larger than reductions |
| followed indirections | 99,346 | only about 2% of resolve calls follow an indirection |
| max resolve chain | 4 | deep indirection chains are not the current bottleneck |
| shortcut hits | `selector_pair_field` 24; `identity_alias_chain` 5 | existing narrow shortcuts barely fire here |

Top reduction heads: `B` 56,409; `C` 36,472; `C'` 25,246; `P` 20,309;
`C'B` 20,142; `K` 14,202; `S` 13,980; `Z` 13,932; `O` 8,986; `A` 8,863.

Static self-host comb shape: 661,784 bytes; 262,025 parse nodes; 146,725
application nodes; 104,562 primitive nodes; 3,226 labels. The hot path is
still reducer/spine/update mechanics, not IO/FFI coverage.

## F2 Status

| item | state |
|---|---|
| evaluator-owned WHNF spine loop | implemented with reusable `EvalSpine` |
| strict-result markers | implemented for Int, Int64, Float64, Float32, ByteString, comparisons/orderings, mixed Int64 shifts, conversions |
| strict WHNF frames | implemented for `seq`, `IO.strict`, and `isInt` paths |
| direct scalar/bytes helpers | route through `reduce_whnf_from`; old side mini-drivers removed |
| generic strict coercions | share `eval_whnf_value`; no separate helper driver shape |
| remaining F2 work | app update/rebuild mechanics and repeated resolve/classification |
| GC/F5 unblocker | closer, but not done; the runtime still needs an explicit enumerable root protocol for every path before GC |

## Lessons From `eval.c`

| lesson | Rust status | next action |
|---|---|---|
| compact node representation | not started | defer until evaluator/GC shape settles |
| primitive heads as tags | partial | extend only when profiles justify it beyond current `KnownPrim` |
| application spine on evaluator stack | partial | current `EvalSpine` is a bridge; final C shape wants less rebuild |
| redex-root update and app reuse | partial | current priority: reduce extra-argument app rewrite/rebuild cost |
| Y knot | done | keep cycle/no-self-indirection behavior |
| permanent common nodes/small ints | partial | useful for graph size, not yet a clear speed lever |
| marker continuations for strict forcing | mostly done for F2 common cases | remaining issue is not helper recursion, it is update/rebuild mechanics |
| arity-shaped combinator rewrites | partial | keep hot `B`/`C`/`C'`/`P`/`C'B` paths simple and direct |
| GCRED-style simplification | not started | defer until GC exists |
| IO as graph rewrites | partial | only add more shortcuts with profile evidence |
| counters near evaluator | partial | current counters point at app rewrites and resolve/classification |

## Current Theory

The remaining Rust/C gap is evaluator throughput, not parity noise. The short
self-host proxy is sink-clean and stable at 297,417 reductions, but Rust still
pays for repeated resolve/node classification, owned spine construction, extra
argument rebuild, and transient app allocation where C stays in one stack/goto
loop and mutates the redex/root cells directly.

The newest profile counters make generic argument materialization look
secondary: 8,523 materializations over 37,356 nodes versus 226,403 app rewrites
carrying 1,300,478 extra arguments. The next useful F2 work should therefore
target app update/rebuild mechanics and resolve/classification volume, not
another narrow cache or helper-unification pass.

## Current Next Items

| item | status |
|---|---|
| F2 app rewrite/update work | active |
| self-hosting parity | blocked on runtime throughput/GC; full Rust compile still exceeds 900s |
| GC/F5 | next structural frontier after F2 root/update/stack shape is good enough |
| JS full parity | pending compiler-emitted metadata or generated glue path |
| rare high-level library tests | defer until compiler self-host path is more useful |
| keep compiler Haskell | ongoing; scope is the C runtime/evaluator rewrite |

## Do Not Reapply Blindly

These were measured and either regressed important rows or failed the self-host
proxy: start-node resolve-chain compression, saturated-redex root updates,
descriptor-slice `EvalSpine` access, inline `StepAction` marker requests,
all-Int/unrestricted marker probes, broad lazy generic primitive cascades,
primitive singleton/cache seeding, reverse-free inline spine layout, and
skipping outer-app rethreading after reduction.
