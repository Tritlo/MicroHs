# Math services in the standalone WAT runtime

`services-math.wat` implements the floating-point services that
`lib/Data/Double.hs` and `lib/Data/Float.hs` import with `foreign import
ccall`. The whole implementation is WAT. It needs no C or Rust code and no
host math import. It runs in Wasmtime and in a browser.

## Interface

`$service_math(id, args)` returns a boxed Double or Float node, or zero when
the service id is not a math service. The arguments are node pointers in weak
head normal form. The function never raises an exception, never evaluates,
and never collects. The argument order is the C order.

| Service | Signature | Method |
| --- | --- | --- |
| `sqrt`, `sqrtf` | x | native `f64.sqrt`, `f32.sqrt` |
| `scalbn`, `scalbnf` | x, n | exact power-of-two scaling in at most three steps |
| `exp`, `log` | x | fdlibm `e_exp.c`, `e_log.c` |
| `sin`, `cos`, `tan` | x | fdlibm `k_sin.c`, `k_cos.c`, `k_tan.c` with `e_rem_pio2.c` and Payne-Hanek `k_rem_pio2.c` |
| `atan`, `asin`, `acos` | x | fdlibm `s_atan.c`, `e_asin.c`, `e_acos.c` |
| `atan2` | y, x | fdlibm `e_atan2.c` |
| `pow` | x, y | fdlibm `e_pow.c` |
| `expf` ... `powf` | Float | the Double function, then one `f32.demote_f64` |

## Provenance

The Double algorithms are ports of the Sun fdlibm / FreeBSD msun sources in
the form used by musl libc and by the rust-lang/libm crate (MIT). The
coefficients are copied exactly. Each function in the WAT file names its
source file and carries the Sun permission notice. `scalbn` follows the
generic algorithm from rust-lang/libm. The IEEE special cases are the ones
the sources implement, which match C Annex F. Wasm has no floating-point
exception flags, so the flag-raising expressions from the sources are
omitted.

The WASI build of the C runtime uses wasi-libc, which is musl. musl's
`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, and `scalbn` are the
same algorithms as here. musl's current `exp`, `log`, and `pow` are the
table-driven ARM optimized-routines versions, so those three can differ from
the C runtime by one ulp in rare cases.

## Accuracy

Measured with a temporary harness against MPFR at 300 bits. Every call went
through `$service_math` with boxed arguments. Three random seeds gave 90,000
random cases per function, plus about 50 edge values per function and a set
of hard argument-reduction cases (nearest Doubles to `k * pi/2` for random
`k` up to 2^62, and the 2^849 worst case). The table shows the maximum over
all runs. "Not correctly rounded" is the share of results that differ from
the correctly rounded value.

| Function | max ulp error | not correctly rounded | special-case mismatches |
| --- | --- | --- | --- |
| `sqrt`, `sqrtf` | 0.5 (correctly rounded) | 0 | 0 |
| `scalbn`, `scalbnf` | 0.5 (exact, one rounding for subnormal results) | 0 | 0 |
| `exp` | 0.85 | 7.1% | 0 |
| `log` | 0.78 | 2.4% | 0 |
| `sin` | 0.76 | 2.2% | 0 |
| `cos` | 0.73 | 2.0% | 0 |
| `tan` | 0.76 | 2.4% | 0 |
| `atan` | 0.73 | 0.4% | 0 |
| `asin` | 0.79 | 4.8% | 0 |
| `acos` | 0.85 | 6.0% | 0 |
| `pow` | 0.81 | 6.8% | 0 |
| `atan2` | 1.33 | 8.5% | 0 |
| all Float functions | 0.50 | 0 in 90,000 per function | 0 |

`atan2` is the only Double function that exceeds 1 ulp. The fdlibm method
rounds `y/x` before it calls `atan`, and that rounding adds up to about 0.5
ulp. This is the behavior of musl and of the C runtime on WASI. The Float
functions are correctly rounded in every sampled case because the Double
result carries at least 29 extra bits; a Float rounding error remains
possible when the true value lies within about 2^-29 ulp of a rounding
boundary.

The sampled maxima are not proofs. The fdlibm error analysis bounds each
Double function to below 1 ulp except `atan2`. The special-case checks cover
signed zeros, infinities, NaN, subnormal inputs, `pow(x, 0)`, `pow(1, y)`,
`pow(-1, +-inf)`, negative bases with integer and non-integer exponents,
overflow and underflow thresholds, and `scalbn` with exponents up to the
`Int` limits.

## Memory

The fragment owns `0xe800..0xec70` in the static range. `0xe800` holds 66
words of `2/pi` from a data segment. The rest is scratch that Payne-Hanek
reduction writes and reads within one call. `$math_rem_pio2` returns its
double-double remainder through `0xec60` because wasm-as 108 does not
assemble stack-style use of multi-value results.
