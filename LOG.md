# WASM evaluator and foreign-call experiment

## 2026-09-14 00:47 CEST — scope and baseline

- Branch: `mhs-wasm`, from `f62d45863cf9b639cf19b8ae6f6444f76db8b141`.
- Preserve the preceding GHC FFI log and the pre-existing `mhs.conf` and `rust_out` files.
- Host: x86-64 Linux on WSL2. Wasmtime 46.0.1 and Node 22.22.1 are installed.
- Implement direct WASM foreign calls and investigate a graph evaluator written in WAT.
- Run the self-hosting compiler through WASI. Reuse the browser WASI shim where practical.
- Compare current C and Rust builds, both native and WASM. Record build flags, memory settings, output identity, and run order.
- Use full compiler output for correctness. Separate compilation and startup from execution when measuring engines.
- A speed improvement is a hypothesis. Handwritten WAT still passes through an engine compiler.

### Intended stages

1. Build fresh baselines and establish a common self-host input and output.
2. Prove direct scalar WASM imports through the compiler and terminal runner.
3. Write graph traversal and common reductions in WAT. Initially retain compiled support for allocation, GC, I/O, and uncommon primitives. Label this stage as a hybrid evaluator.
4. Extend WAT coverage and compare dispatch designs using full self-host output checks.
5. Verify browser execution and document the tested interface and remaining limitations.

### Progress and measurements

- No performance measurements yet. Existing README numbers are historical controls only.
- The shared board has no matching `microhs`, `microhs-rust`, or `wasm` notes.
- Requested design advice from the Magi council, including Claude Fable.
- Automatic approval review rejected repository inspection by the external advisors. Retried with an abstract design question that excludes source and workspace access.

## 2026-09-14 00:49 CEST — design review and WASI build repair

- Fable and Grok completed the abstract review. Both recommend a dense `br_table` first and static WASM linking. Tail-call dispatch remains an experiment.
- Use a trampoline boundary for the first reducer. WAT returns before GC, scheduling, or unsupported work. It writes the stack and allocation state before return. The compiled runtime then resumes at the same graph node.
- Keep the current C node layout for the first comparison. A new representation would confound the effect of the handwritten reducer.
- The Rust WASI build failed because `NO_THREAD` was excluded on WASI although the scheduler uses it. Removed that incorrect conditional declaration. Native behavior is unchanged.

## 2026-09-14 00:58 CEST — first controls and linked WAT loop

The benchmark reads an immutable `mhs`/`src`/`lib` snapshot from the branch base. An earlier native C run read source files while they changed; that run is excluded. The input compiler has 675,165 bytes and SHA256 `4c5adefd0b0afef06d77867448405336ed470ab7ffffe5571b32a01aec4e9cb2`.

| Runtime | Parse and evaluate | Process wall | Maximum RSS | Output |
| --- | ---: | ---: | ---: | --- |
| Native C, GCC 13.3.0 `-O3` | 50.999 s | 51.07 s | 807,360 KiB | Exact input match |
| Native Rust 1.97.1, release | 60.891 s | 60.90 s | 371,096 KiB | Exact input match |
| C WASI, SDK Clang 21.1.8 `-O3`, Wasmtime 46.0.1 | 83.913 s | 84.45 s | 741,360 KiB | Four literals differ from native C |
| Rust WASI 1.97.1, release, Wasmtime 46.0.1 | 80.818 s | 81.01 s | 887,684 KiB | Exact input match |

- These are single runs in the listed order. No other builds ran during the measurements.
- C uses 50,000,000 nodes: 16 bytes per native node and 12 bytes per WASM node. Rust uses 8-byte cells and `MHS_GC_NODE_INTERVAL=33554432`. These controls do not impose equal memory limits.
- Native Rust: 128 GCs and 21.455 s of GC pauses. Rust WASI: 128 GCs and 26.822 s of GC pauses.
- C-WASM serializes two `2147483648` literals as `-2147483648`. Two floating-point constants also differ. Investigation is in progress. Do not normalize these outputs or claim cross-target byte identity.
- Artifacts and immutable source snapshot: `/tmp/mhs-wasm-baseline/`.
- `wasm/reducer.wat` implements application traversal, indirection compression, bitmap allocation, and S/K/I/B/C/A/Y reductions. It contains no function imports. Other work returns to C.
- `wasm/build.sh` compiles and statically links the support runtime and WAT module. The new WASI configuration uses real WebAssembly exceptions for `setjmp`/`longjmp`.

## 2026-09-14 01:02 CEST — first complete WAT self-host and direct imports

- The WAT hybrid compiled the frozen compiler sources and matched C-WASM output byte for byte: 675,167 bytes, SHA256 `dd25f3e205307c6067e95bbe7e1b3ea9271fd1a3d77d962bebacdb2e3b72b8dc`.
- C-WASM also reproduced that output when run from its own output compiler. The WASM target reaches a fixed point. Its native-output differences remain under investigation.
- The first WAT run measured 91.784 s internally and 92.37 s wall, with 737,760 KiB maximum RSS. Other correctness checks and builds ran concurrently. This is a diagnostic run, not an accepted speed comparison.
- Inspection showed that the raw WAT retained calls to argument and allocation helpers. The build now inlines those helpers with Binaryen before linking. The resulting reducer contains one function and no function imports. The compiled C support is unchanged by this step.
- `foreign import wasm "module export"` is implemented. The Wasmtime sample passed i32, i64, f32, f64, and `IO ()` calls. The i64 result `9007199254741003` crosses directly without JavaScript number conversion.
- Eighteen temporary frontend checks passed. They cover malformed syntax, unsupported types and exports, unsupported output modes, conflicting signatures, compatible signed/unsigned aliases, and symbol identity.

## 2026-09-14 01:13 CEST — complete combinators and first speed improvement

- Extended WAT to all 23 core combinators, including partial-arity rules, T3 through T16 tuples, and TAG0 through TAG32 constructors.
- Repaired `imath.c`: the one-digit division remainder is unsigned. `mp_int_set_value` converted remainders above `INT32_MAX` to negative values on WASI. Use `mp_int_set_uvalue`.
- The repair makes `7000000000 quotRem 4000000000` return remainder `3000000000`, instead of `1294967296`. Floating-point probes now agree with native C.
- Corrected C-WASM self-host and fixed point pass. Its output has 675,167 bytes and SHA256 `ddf41a0b2cf490576aefc68fcc193c4e4642503961e6380f4789fba7bc2e828b`. Only two `0x80000000` literals differ from native output, as signed 32-bit encodings.

First controlled AOT pair, with the corrected runtime and no other builds:

| Runtime | Parse and evaluate | Process wall | Maximum RSS | Output |
| --- | ---: | ---: | ---: | --- |
| C-WASM | 79.491 s | 79.57 s | 622,656 KiB | Exact corrected oracle match |
| WAT v2 hybrid | 54.392 s | 54.46 s | 622,464 KiB | Exact corrected oracle match |

- WAT v2 is 31.6% faster in this pair. Both use the same 50-million-node, 12-byte C heap. This is one pair, not a median.
- Both modules were compiled with `wasmtime compile -W exceptions=y -O opt-level=2`. AOT compilation is outside the timed process. Runs use `--allow-precompiled` and an 8 MiB WASM stack limit.
- Independent review found one slice-accounting difference for an indirection that ends at an application. WAT now performs the C `T_AP` step directly and consumes the same slice count. That change follows this measurement and requires a new run.
- Chromium 151 passed direct WASM imports, merged C/WASI, and merged WAT/WASI. All produced the same scalar/IO sample output.
- The browser adapter also passed Unicode arguments, paths and stdin, split UTF-8 output, binary file output, stderr, and exit status. It corrects an argument-size bug in the existing WASI shim.
- Use explicit Binaryen feature flags. `--all-features` in Binaryen 132 can emit compact imports that Chromium 151 rejects. Binaryen 124 also needs multimemory enabled while resolving the reducer's memory import during linking.

## 2026-09-14 01:24 CEST — scheduling parity and rejected allocator cache

| Probe | Parse and evaluate | Process wall | Maximum RSS | Output |
| --- | ---: | ---: | ---: | --- |
| WAT v3, direct IND-to-AP step | 51.675 s | 51.74 s | 622,656 KiB | Exact corrected oracle match |
| WAT v4, allocator fields in private mutable WASM globals | 54.497 s | 54.57 s | 622,464 KiB | Exact corrected oracle match |

- The allocator cache is rejected. Entry/exit synchronization and global access cost outweighed the intended reduction in memory aliasing. Restored v3 source.
- A temporary 305-case fixture exercises all core combinators and their partial arities, all tuple/tag constructors, and an indirection-to-application case. C and WAT produced identical 915-line output, including `(13099,8384)` for allocation and reduction counts.
- `wasm/bench.sh` now records frozen source/input hashes and alternates run order. It checks exact output after every run and keeps all results in a new directory.
- Requested another abstract Fable consultation about integer fast paths, continuation frames, and dispatch. No repository contents were supplied.

## 2026-09-14 01:56 CEST — measured arithmetic experiments

Fable recommended measuring exits before extending arithmetic. The v3 profile
recorded 3,477,936,061 WAT steps and 360,183,839 returns to C. There were
110,227,343 binary integer exits, of which 84,853,119 had evaluated operands
after indirections. Other large groups were 65,092,092 integer values and
77,080,862 foreign-pointer values.

| Probe | Parse and evaluate | Output |
| --- | ---: | --- |
| WAT v5, ready integer operations through indirections | 53.974 s | Exact corrected oracle match |
| WAT v6, direct integer values only | 52.569 s | Exact corrected oracle match |
| WAT v7, direct integers plus WHNF value handoff | 51.361 s | Exact corrected oracle match |

- These are individual AOT runs. The v7 result is effectively level with v3 until repeated runs establish a difference.
- Return bit 0 selects the C WHNF continuation. This avoids repeating the C top-loop step after WAT has produced a value. Node pointers remain aligned to four bytes.
- Integer fast paths preserve checked signed overflow and division errors. They return before mutation when C must handle an exception or an unsupported case.
- The broader v5 path exposed an evaluation-order bug: resolving a cyclic left indirection could prevent a right-side exception. The corrected path inspects the right operand first. V6 uses only direct values and leaves indirections to C.
- V6 passed 414 arithmetic cases and 305 core cases with exact values and allocation/reduction counts. Another 105 boundary assertions passed. The right-side exception reproducer also matches C.
- Full Chromium 151.0.7922.34 self-host passed for corrected C-WASI and WAT v3. Both produced the exact corrected oracle. Diagnostic times were 53.934 s and 49.544 s respectively, with concurrent work. These are browser compatibility checks, not final speed comparisons.
- Public GitHub metadata confirms `Tritlo/MicroHs` is public. Requested a bounded Fable review of the task's runtime, WAT source, and build script after establishing that fact. No configuration files, credentials, or unrelated log contents were included.

## 2026-09-14 02:09 CEST — stronger controls and code generation

| Control | Parse and evaluate | Process wall | Maximum RSS | Output |
| --- | ---: | ---: | ---: | --- |
| C-WASM with whole-module Binaryen `-O3` | 78.673 s | 78.74 s | 622,464 KiB | Exact corrected oracle match |
| WAT v7 with whole-module Binaryen `-O3` | 56.300 s | 56.38 s | 622,080 KiB | Exact corrected oracle match |
| Native Rust, GC interval 78,643,200 | 50.058 s | 50.07 s | 753,948 KiB | Exact native oracle match |
| Rust-WASM AOT, GC interval 33,554,432 | 80.074 s | 80.18 s | 774,144 KiB | Exact native oracle match |
| Rust-WASM AOT, GC interval 78,643,200 | 70.443 s | 70.62 s | 1,561,344 KiB | Exact native oracle match |

- The larger Rust interval reduces collections from 128 to 55. Native GC time is 12.095 s; WASM GC time is 16.690 s.
- Whole-module Binaryen optimization is not the default. It slowed the WAT variant. Disassembly confirms that it retained the reducer call boundary, so inlining the whole reducer into C does not explain this result.
- Fable completed the bounded source review and recommended inspecting register allocation and classifying WHNF continuations before adding more fast paths.
- The AOT disassembly shows that v3 keeps the stack base in a register during application traversal. V7 reloads it from a native stack slot for each application. Both versions also reload the stack limit.
- The next probe removes the separate WAT return-flag local. It encodes the flag in the returned node only at the exit. This keeps the existing low-bit ABI.

## 2026-09-14 02:16 CEST — final hybrid selection and expanded target

- WAT v8 removed the separate return-flag local. It measured 54.970 s and matched the corrected C-WASM oracle. This did not improve on the simpler v3 reducer.
- Restored v3 as the hybrid control. Removed the integer fast paths and WHNF-return flag. Retained optional step/exit profiling without the argument-readiness walker.
- The rebuilt hybrid passed 305 differential cases, exact allocation/reduction counts, and 31 slice/stack/free-node assertions. Artifact SHA256: `e02349ac87cb19f62fad29f80ca4616efc1490184aef8e324fd7b175db437b87`.
- The current frontend reached stage2=stage3 under both C/WASI and the rebuilt WAT hybrid. Both emitted the same 680,865-byte compiler: `cff58f848a7580a24ed75450ec82edad9d753bb6ab8d6e023d6daa2eae256336`.
- Both self-hosted compilers emitted identical C for the direct-WASM example. The linked example produced all seven expected lines. Evidence: `/tmp/mhs-wasm-frontend-bootstrap/README.md`.
- The requested target now includes the full runtime in WAT, with no C compilation in its build. General C FFI can be dropped. Foreign calls will use JS or WASM. Source documentation must describe interfaces and invariants for later review.
- Keep the hybrid as a verified control. Start the standalone runtime under `wasm/standalone/`. Do not describe this new runtime as complete before it passes its acceptance checks.

## 2026-09-14 02:38 CEST — standalone contracts and Haskell Integer backend

- `ABI.md` specifies node memory, payload memory, root locations, allocation rules, continuation frames, and module interfaces.
- Strict arguments and exception handlers use explicit continuation frames. The evaluator does not recurse through the engine call stack. All suspended graph references remain visible to the collector.
- Fable reviewed this structure and is implementing an independent exact decimal floating-point parser. Its first delegate tool call timed out; the underlying work continued. The source file is present and remains under numerical verification.
- The existing `lib/no-gmp` implementation removes all 23 bignum foreign symbols and all MPZ literals. This reuses Haskell arithmetic instead of adding a new WAT bignum library.
- No-gmp current compiler seed: 688,164 bytes, SHA256 `4691c8307cfb1351e2b281a88ca2c812c0bd7b82dac87a8f161a12abc0b280dd`.
- Corrected C/WASI and WAT each reached stage2=stage3: 688,166 bytes, SHA256 `c6e6b0747d17f696b518853d833966cacaf19c2b2523d70fb082379aae497962`.
- Native C reproduced the no-gmp seed exactly. Any new performance comparison must use this same compiler and library selection on every evaluator.
- A pre-existing native64-only no-gmp defect remains: conversion of Int/Word values above 32 bits can violate its fixed 16-bit-digit invariant. The verified compiler workload does not trigger it. The wasm32 target does not have those machine Int/Word values. This is recorded as a separate finding; no unrelated library repair is included.
- Standalone loader/name probes pass, including a 307-case C serialization differential and the full no-gmp compiler. Heap probes cover roots, cycles, interior pointers, free-list reuse, bitmap reclamation, and overflow checks. These component checks do not yet establish a complete standalone runtime.

## 2026-09-14 03:08 CEST — first complete standalone compiler run

- The standalone WAT integration build compiled the no-gmp compiler sources successfully. No compiled C or Rust support is present in this module.
- Output: 688,166 bytes, SHA256 `c6e6b0747d17f696b518853d833966cacaf19c2b2523d70fb082379aae497962`. This exactly matches the corrected C/WASI and hybrid references for the same no-gmp workload.
- Input: `/tmp/mhs-wasm-no-gmp/wat/stage3.comb`. Source: `/tmp/mhs-wasm-no-gmp/source/`. Output: `/tmp/mhs-wasm-no-gmp/standalone-stage.comb`.
- The integration build temporarily rejects math services while the final math component is under development. The compiler run did not invoke one of those unavailable services. This is a full compiler execution result, not a claim that every runtime operation is complete.
- The runtime includes handwritten graph traversal, allocation, collection, finalizers, numeric operations, bytestrings, arrays, explicit strict continuations, exception handling, full graph forcing, MD5, and WASI file services. The no-gmp Integer library remains Haskell.
- Component checks include 8,962 numeric oracle cases, 290 MD5 algorithm comparisons, byte/array differentials, 100,000-level graph forcing, and file/encoding/directory checks in Wasmtime and Chromium.
- Review found and fixed a digest service argument-order mistake that algorithm-only tests did not cover. All digest bindings take input first and output second. Keep service-level checks as well as algorithm checks.
- The user explicitly requested the best of both existing runtimes. Rust references supplied the MD5 structure, iterative traversal patterns, and missing narrow-memory service coverage. C supplies useful graph and serialization contracts. A reference implementation's bug is not a requirement.
- WAT preserves Word-to-Word64 zero extension and complete 64-bit foreign scalar payloads. Existing C-WASM implementations of some corresponding operations truncate or sign-extend incorrectly. These discrepancies require targeted evidence rather than byte-output normalization.
- A standalone fixed-point run and direct no-C JS/WASM frontend integration are next. Final performance comparisons remain pending.

## 2026-09-14 03:52 CEST — standalone fixed points and WASM-specific experiments

- The first standalone no-gmp compiler reached a fixed point in Wasmtime. Chromium produced the same 688,166-byte output, SHA256 `c6e6b0747d17f696b518853d833966cacaf19c2b2523d70fb082379aae497962`.
- The updated frontend also reached stage2=stage3: 696,922 bytes, SHA256 `309c5b6884f57c67c2f41a5945da3585183cb59a75969d291835e747262955c3`. Its self-hosted `.wat` output matched the native GHC compiler's bindings and metadata. Direct WASM and JS examples passed Wasmtime and Chromium without generated C.
- A diagnostic profile of the first standalone workload recorded 65.76 s process time, 21.701 s collection time, 93 collections, and 4,368,134,671 node allocations. The mean live set at collection was about 3.03 million nodes. These measurements used the earlier frozen integration module and concurrent development work.
- Skipping the collector work queue for raw payload blocks with pointer mask zero produced the same compiler output in 61.43 s. This is a candidate, not an established speedup; the previous unprofiled diagnostic was 62.86 s.
- The user requested experiments that use WASM capabilities directly. The current 12-byte graph layout is a correctness baseline, not a permanent design constraint for standalone WAT.
- A tail-call experiment passed the complete compiler byte check but took 77.57 s. It passed six state values through `return_call` after each reduction. This variant is rejected. Tail calls are not automatically cheaper than a loop.
- An isolated eight-byte-cell experiment is next. It stores rare Int64/Double payloads separately and uses a descriptor-pointer bit for the byte-string kind. An independent WasmGC experiment will assess engine-managed graph allocation, cyclic graphs, and weak-pointer limitations.
- `wasm/standalone/bench.sh` now freezes one no-gmp source/input workload for native C, native Rust, C-WASM, Rust-WASM, standalone WAT, and optional hybrid controls. It records hashes, explicit Rust collection intervals, process wall time, RSS, and exact output matches.

## 2026-09-14 04:06 CEST — compact runtime integration

| Diagnostic variant, earlier no-gmp compiler | Process wall | Maximum RSS | Output |
| --- | ---: | ---: | --- |
| 12-byte nodes, 50 million, skip raw-block marking work | 61.43 s | 626,688 KiB | Exact wasm32 oracle |
| 8-byte nodes, 50 million | 55.61 s | 431,040 KiB | Exact wasm32 oracle |
| 8-byte nodes, 75 million | 51.75 s | 629,184 KiB | Exact wasm32 oracle |

- Selected compact eight-byte nodes with 75 million slots. The node arena remains 600,000,000 bytes. Moved its 9,375,000-byte bitmap after the arena to avoid generated foreign-name memory. Int64 and Double use managed raw payloads; byte-string kind uses the aligned descriptor pointer's low bit.
- The shared reducer has a build-time stride constant. The standalone build selects eight bytes before optimization. The C hybrid retains twelve bytes and reproduces its earlier binary hash exactly.
- Compact validation passed 28 state/GC/serialization groups and 8,962 numeric oracle cases. The last node at index 74,999,999 survived collection and serialization checks. Both serializer bitmaps and active/suspended RNF bitmaps passed upper-bound checks.
- Additional checks passed 307 C loader comparisons, 56 print/serialize comparisons, all 286 primitive and 284 fixed-service roundtrips, 20,000-level graphs, malformed/random input, stream encodings, and 13 command checks. These integration tests used the real math implementation.
- Fable completed all 24 handwritten math services. MPFR at 300 bits checked 90,000 random cases per function across three seeds, plus special cases and difficult argument reduction. Sampled errors were below one ulp except fdlibm atan2 (maximum 1.33 ulp). All Float samples rounded correctly. Sampled maxima are not proofs; details are in `wasm/standalone/MATH.md`.
- WasmGC handled typed and cyclic graphs in Wasmtime and Chromium. It did not outperform the temporary eight-byte arena microbenchmark. That arena knows each batch's lifetime and does no general tracing, so its ratios cannot predict complete-runtime speed. Wasmtime copying reclaimed cycles; deferred reference counting did not. Core WasmGC also lacks the weak-reference/finalizer operations needed here. Retained the manual collector. Evidence: `/tmp/mhs-wasm-gc/REPORT.md` and the primary sources linked there.
- Final standalone module SHA256: `f36540007bfbc386a8fdf75f2e6cef1ff384d287a8e7028d8a145b54c3d65227`. Its build uses only Binaryen and WAT source. The final C control is `b77259da4dd7f1d8f89500636a637a20e7e62e94c59b6b2d64cc9fd4ba0fcb9c`; the hybrid is `e02349ac87cb19f62fad29f80ca4616efc1490184aef8e324fd7b175db437b87`.
- Current benchmark input and wasm32 expected output: 696,922 bytes, SHA256 `309c5b6884f57c67c2f41a5945da3585183cb59a75969d291835e747262955c3`. Native C and native Rust independently produced the same 696,920-byte native reference from that exact input: `dffb9ecc2ad0892ab4ac648eea915d773722076c07d6a8ef8cc20a3549264a18`.
- Remaining resource/format limits are explicit: wide scalars share the 1,048,576-entry payload index; C-compatible 16-digit Double serialization is not lossless. These limits do not invalidate the byte-exact compiler results.

## 2026-09-14 04:19 CEST — final browser and foreign-call checks

- The final compact standalone module, SHA256 `f36540007bfbc386a8fdf75f2e6cef1ff384d287a8e7028d8a145b54c3d65227`, compiled the current compiler in both Wasmtime and Chromium 151.0.7922.34. Both produced the exact 696,922-byte wasm32 reference, SHA256 `309c5b6884f57c67c2f41a5945da3585183cb59a75969d291835e747262955c3`.
- Chromium used 328 source files through the browser WASI adapter. It returned exit status zero and the complete output file. Its 59.142 s execution time is diagnostic only; the browser run overlapped other validation work.
- The final runtime passed direct host-supplied WASM imports, statically linked WASM imports, and JS imports in Chromium. Foreign-name serialization and deserialization also passed. Checks preserved `9007199254741003`, unsigned Word64 maximum `18446744073709551615`, Float and Double values, and the bytes `[0,128,255]`.
- Browser evidence is in `/tmp/mhs-wasm-direct/bootstrap/browser-selfhost/wat.json` and `/tmp/mhs-wasm-direct/bootstrap/full-browser-ffi.json`. Bootstrap and direct-FFI commands are documented in `wasm/standalone/BOOTSTRAP.md` and `wasm/standalone/FFI.md`.
- The controlled benchmark uses the final artifacts and three rounds for all six runtimes. No builds or other benchmark processes run concurrently with it. A separate full-compiler inlining experiment follows because explicit Wasmtime inlining improved the small graph probe.

## 2026-09-14 04:26 CEST — first controlled full-runtime comparison

| Runtime | Process wall median | Range | Median maximum RSS |
| --- | ---: | ---: | ---: |
| Native C | 49.02 s | 48.78–49.21 s | 807,360 KiB |
| Native Rust | 50.83 s | 50.33–51.15 s | 765,888 KiB |
| C-WASM | 75.05 s | 74.45–75.86 s | 622,464 KiB |
| Rust-WASM | 71.53 s | 71.33–72.04 s | 1,597,632 KiB |
| WAT reducer with C support | 50.56 s | 50.54–51.03 s | 622,464 KiB |
| Standalone WAT | 51.83 s | 51.64–53.65 s | 629,952 KiB |

- All 18 runs used the same current no-gmp compiler input and source snapshot. All outputs matched their exact native or wasm32 reference. Results, inputs, executables, expected outputs, hashes, and logs are in `target/mhs-wasm/selfhost.DdSU6T/`.
- Standalone WAT used 30.9% less time than C-WASM and 27.5% less than Rust-WASM. It remained 5.7% slower than native C and 2.0% slower than native Rust. These measurements establish a win over both compiled-WASM controls, not over either native control.
- The host was AMD Ryzen 9 7950X, Ubuntu 24.04 under WSL2. Native C used GCC 13.3.0 `-O3`. Rust 1.97.1 used release settings and allocation interval 78,643,200 on both targets. C-WASM used Clang 21.1.8 `-O3` plus Binaryen 124 `-O3`. The hybrid retained its measured faster configuration without whole-module Binaryen optimization. Wasmtime 46.0.1 precompiled all WASM modules at optimization level 2 outside timing.
- Explicit Wasmtime `-C inlining=y` passed the full compiler check but took 52.82 s and 630,336 KiB RSS. It did not improve the default median. The flag must be supplied to both AOT compilation and execution; a mismatched engine configuration rejects deserialization before execution. Evidence: `target/mhs-wasm/selfhost.wWpIL2/`.
- The user requested continued work after accepting the compiled-WASM improvement. The next experiment profiles the compact module and tests direct integer results in existing redex nodes. It remains separate from the verified default until correctness checks and measurements support it.

## 2026-09-14 04:38 CEST — compact collector profile

- The compact runtime allocated 4,389,728,305 nodes during the current compiler run. Only 2,451,395 were boxed Int values. The small-integer cache handles most integer results. Eliminating uncached Int boxes can remove less than 0.1% of total allocations, so this change is deferred.
- In-place results must retain the existing small-integer cache behavior. Weak-key lookup follows indirections to cached nodes, which are permanent. Writing a cached scalar directly into a temporary redex would change this liveness behavior.
- The first compact profile recorded 63 collections and 13.958 s collection time. Its average live set was 3.11 million nodes. The average payload index had about 241,000 entries. Only 137 continuation frames were present across all collection points.
- A second phase profile passed the same compiler byte check and recorded 13.538 s in collection: 13.377 s strong marking, 0.032 s bitmap initialization, 0.00008 s weak marking, 0.033 s finalizers, 0.046 s payload sweep, and 0.050 s free-bit counting. There is no scan over every node in the arena. Bitmap or sweep changes cannot explain a material speed gain here.
- Wasmtime guest sampling also identified node marking and queue insertion as the collector's main work. Payload interval lookup was small. Samples include profiler overhead and do not replace process timing.
- The next isolated probes inline the mark-queue helper and compress indirections in known graph links. Compression must preserve root addresses, conservative scalar words, weak-pointer behavior, and cyclic graphs. No candidate has replaced the verified standalone artifact.
- Profile sources, binaries, logs, exact compiler outputs, and the guest sample are in `/tmp/mhs-wat-final-profile/`.

## 2026-09-14 04:58 CEST — isolated tracing candidates

| Candidate | Process wall, one run | Maximum RSS | Compiler output |
| --- | ---: | ---: | --- |
| Inline the existing mark-queue helper | 57.59 s | 629,568 KiB | Exact reference |
| Publish thread state before GC and at the scheduler boundary | 52.37 s | 629,952 KiB | Exact reference |
| Compress indirections in graph links during GC | 53.16 s | 628,992 KiB | Exact reference |

- None of these single runs establishes an overall gain over the retained 51.83 s median. No candidate has replaced the source implementation.
- Compression changes only known application and indirection links. Generic root and payload words can contain scalar values and remain unchanged. Other roots retain their original node addresses. Floyd cycle detection has no length limit. All 14 compression probes and 28 existing state/layout groups pass.
- The compression profile still passed the full compiler check. It recorded 62 collections, 11.028 s GC time, and an average live set of 2.13 million nodes. The original profile averaged 3.11 million. Total allocation remained about 4.39 billion. Reduced tracing work has not yet established reduced total compiler time.
- A revised compression candidate inlines its initial guard and skips unchanged link stores. Source review found equivalent range, alignment, tag, and null-target behavior. Its optimized diagnostic build passed the same 42 probe groups.
- Separate batch candidates load two or four complete node/header words before processing them. Native code inspection confirms that these used loads occur before the scanner calls. The two-object variant retains both loaded headers in registers; the four-object variant spills two. Each passes 28 state/layout groups and 20 queue-boundary/mixed-object cases.
- The batching experiment follows research on tracing locality. The primary reference is Cher, Hosking, and Vijaykumar, [Software Prefetching for Mark-Sweep Garbage Collection](https://engineering.purdue.edu/~vijay/papers/2004/gc.pdf), ASPLOS 2004. Its results do not imply a large speedup for this runtime. These WAT variants use ordinary loads, not a prefetch instruction.
- A fresh comparative round is running under `target/mhs-wasm/selfhost.xudI62/`. It includes the unchanged baseline, both compression versions, and batches of two and four. Every output must match the same frozen compiler reference. Apparent improvements require repeated confirmation before adoption.
- Candidate sources and component reports remain under `/tmp/wat-gc-compress/` and `/tmp/wat-gc-batch/`. The pre-existing `weak_key_live` limitation on a pure indirection cycle used as a weak key remains separate; ordinary weak keys and application cycles passed the component checks.

## 2026-09-14 05:09 CEST — comparative tracing round

| Runtime | Process wall | Maximum RSS | Compiler output |
| --- | ---: | ---: | --- |
| Unchanged standalone baseline | 52.28 s | 629,952 KiB | Exact reference |
| Indirection compression | 50.02 s | 628,800 KiB | Exact reference |
| Compression with common-path changes | 48.28 s | 628,416 KiB | Exact reference |
| Two-object batches | 51.09 s | 627,840 KiB | Exact reference |
| Four-object batches | 48.62 s | 627,840 KiB | Exact reference |

- All five runs used the same frozen source/input workload and AOT settings. Results are in `target/mhs-wasm/selfhost.xudI62/`. The two strongest candidates are close enough to require repeat measurements. No native speed win is established from this round.
- A combined four-object/compression candidate is under review. Compression can rewrite an indirection whose words a batch already loaded. The scanner must reload that node's current word. Application and payload-header snapshots remain unchanged by the compression helper.
- Each standalone batch candidate also passed three explicit strong graph/payload cycle probes, for 23 batching-specific cases plus the 28 state/layout groups.
- The weak-key indirection-cycle case is being reproduced with a host timeout. A collector must not silently assume that every cyclic graph is invalid. Any Haskell-level reachability claim needs a source reproducer, separate from the graph API case.

## 2026-09-14 05:21 CEST — cyclic weak keys and confirmation inputs

- Weak collection timed out for rooted and unreachable self-IND keys through three routes: direct graph construction, accepted `_0 :0` bytecode, and the existing reducer's `Y I` rules at a slice boundary. The last route constructs the runtime representation of `fix id`. This is stronger evidence than a manually corrupted graph. A complete Haskell scheduling reproducer was not compiled.
- Replaced the unbounded weak-key walk with the cycle-safe link resolver followed by a mark-bit check. Lookup marks nothing. It retains strongly reachable divergent keys and clears unreachable ones. All six timeout cases now complete.
- Six further groups cover weak fixed-point discovery, multiple cycle keys, stable/raw roots, nonresurrection, finalizer execution once, and cached integers. The correction also passed the existing state/layout and compression checks. Evidence is in `/tmp/wat-weak-cycle/REPORT.md`.
- All three confirmation candidates include the weak-key correction. The combined candidate reloads an IND word before using it because another trace can shorten its chain after the batch loads its header. It passed 14 compression, 28 state/layout, 24 queue/snapshot, and six weak-cycle groups, plus all six timeout reproducers.
- Final candidate builders and corrected binaries are in `/tmp/wat-gc-final/`. The combined candidate passed a complete compiler run in 49.36 s before repeated confirmation.

## 2026-09-14 05:36 CEST — final GC selection

| Runtime | Process wall median | Range | Median maximum RSS |
| --- | ---: | ---: | ---: |
| Native C | 50.22 s | 48.46–50.40 s | 807,552 KiB |
| Native Rust | 52.54 s | 50.65–52.63 s | 765,884 KiB |
| Previous standalone WAT | 53.59 s | 53.54–53.76 s | 629,952 KiB |
| Corrected fast compression | 50.10 s | 49.74–50.12 s | 628,608 KiB |
| Corrected four-object batches | 53.02 s | 51.94–53.04 s | 627,648 KiB |
| Corrected compression and four-object batches | 48.82 s | 48.32–50.14 s | 627,840 KiB |

- All 18 outputs matched their exact references. All runs used the same frozen compiler/source workload, explicit Rust allocation interval, and AOT settings. Round two reversed runtime order. Results and copied inputs/binaries are in `target/mhs-wasm/selfhost.080E62/`.
- Selected combined tracing. Its median is 8.9% below the previous WAT collector, 2.8% below native C, and 7.1% below native Rust in this confirmation. Native C and WAT ranges overlap, so the C comparison is a modest median advantage.
- Against the earlier compiled-WASM controls on the identical workload, the final WAT median uses 35.0% less time than C-WASM (75.05 s) and 31.7% less than Rust-WASM (71.53 s). Those control samples are in `target/mhs-wasm/selfhost.DdSU6T/` and were not repeated during this GC-only confirmation.
- Adopted the combined heap and the weak-key correction. Removed the false source assumption that valid graphs cannot contain pure IND cycles. Heap, ABI, and thread documents now describe batch coherence, chain shortening, fixed root addresses, and weak-key cycle behavior.
- The public `wasm/standalone/build.sh` output matches the winning benchmark artifact byte for byte: SHA256 `6c140f94c75005704b0e07ad8e68f989beef90fe2d4de83eb5aaa7325e831a89`. It still assembles and links only WAT with Binaryen. Final browser and direct-FFI checks on this artifact are running.

## 2026-09-14 05:45 CEST — final acceptance complete

- The selected public module completed full Chromium self-hosting. Output is exactly 696,922 bytes, SHA256 `309c5b6884f57c67c2f41a5945da3585183cb59a75969d291835e747262955c3`. It matches the Wasmtime reference and the repeated benchmark outputs.
- Rebuilt all direct-import and serialization modules from the adopted source with Binaryen 124. Wasmtime passed linked and host-supplied WASM imports plus foreign-function and wide/binary serialization. Chromium passed those cases and JavaScript imports. Exact i64, unsigned Word64, Float, Double, and binary bytes remain intact.
- Final verification and module hashes are in `/tmp/mhs-wasm-direct/final-gc/verification.json`. The tested runtime hash is `6c140f94c75005704b0e07ad8e68f989beef90fe2d4de83eb5aaa7325e831a89`. All three harnesses exited zero and closed their servers and browsers. No source changes were needed after acceptance.
- The build scripts pass shell syntax checks. The final diff passes whitespace checks. The source and documentation are ready on `mhs-wasm`; no commit, push, or deployment was performed.

## 2026-09-14 22:51 CEST — deployment review

- Reviewed `50b41da2` on `mhs-wasm-deployment-review`. The Orbitorio spike is still in progress. This review covers the general runtime and browser adapter. It does not certify a game deployment.
- Fixed an unbounded indirection walk in the reducer. An ordinary Haskell child evaluating `loop = loop` previously prevented its parent from resuming. Indirection traversal now consumes the scheduling slice. The parent resumes and can kill the child. A 1,000-link terminating chain also resumes correctly across 143 calls with a seven-step allowance.
- Fixed another unbounded indirection walk in serialization. A divergent field left by that child now raises the catchable `Serialize` exception. A cyclic list still serializes and deserializes correctly.
- Fixed the use of WASI errno values as Haskell runtime exception codes. Directory reads now reach the existing `IOError` handler and continue, as in C-WASI. Unsupported codecs and output failures use fatal diagnostic 94 and retain errno. A codec failure previously entered a malformed `SomeException` handler and could return status zero without completing the program. Injected write and flush failures now return status one. These fatal errors are not catchable in Haskell.
- The browser adapter now validates all `fd_read` and `fd_write` buffers and result pointers before transfer. Invalid buffers previously caused truncated successful writes or partial reads before a host exception. All twelve vector-I/O probes now pass, including valid and empty transfers and an invalid later vector.
- JavaScript imports now reject asynchronous results from other realms and thenables. Rejection handlers prevent a second unhandled Promise rejection. Four asynchronous-return probes and three synchronous unit-return probes pass. UTF-8 arguments and environment values also pass.
- Eighteen existing tests match their golden files. `Weak` retains its pre-existing finalizer-output ordering difference; its value and finalizer checks pass. The selected tests cover arrays, byte strings, text, foreign and stable pointers, exceptions, threads, arithmetic, and MD5. The hybrid C/WAT build and arithmetic smoke check also pass.
- The corrected WAT runtime completes the frozen compiler workload with exact output: 696,922 bytes, SHA256 `309c5b6884f57c67c2f41a5945da3585183cb59a75969d291835e747262955c3`. Its runtime SHA256 is `0bd0687ecd0557c63595b5a26b84b90408fc71ed1e580931c67a868c75577d05`. Linked and host-supplied foreign calls, wide scalars, binary bytes, and foreign-function serialization pass. Strict TypeScript and shell syntax checks pass. These are correctness checks, not new performance measurements.
- Remaining deployment work includes a measured memory budget, host cancellation and worker lifecycle, and standalone WAT CI coverage. Initial linear memory is fixed at 640 MiB and can grow to 2 GiB. The payload base is fixed, so changing only the memory declaration is insufficient. `runWasi` executes synchronously; its timer fallback occupies the current JavaScript thread. CI currently builds the Rust browser runtime, but does not assemble or test standalone WAT.
- Temporary reproductions and results are in `/tmp/mhs-wasm-deployment-review-20260914`, `/tmp/mhs-meta-audit`, and `/tmp/mhs-deployment-audit`. No tracked test files were added.
- Chromium 151.0.7922.34 also passes all twelve vector-I/O probes, UTF-8 arguments and environment values, and asynchronous-return rejection with no unhandled rejections. The standalone WASM and JavaScript examples each pass twice with exact scalar and IO output.
