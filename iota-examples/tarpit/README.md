# Turing-tarpit long reductions

Animating "long reductions" from Hemann & Holk, *Visualizing the Turing Tarpit*
([ACM 10.1145/2661103.2661105](https://dl.acm.org/doi/10.1145/2661103.2661105);
code & data: [github.com/tarpit/jot-code](https://github.com/tarpit/jot-code)),
in the ι-tree morph style of [`../../iota`](../../iota).

## How a Jot program becomes an ι-tree

A **Jot** program is a binary string. Its semantics build a combinator term:

    [[ε]] = I        [[F0]] = [[F]] S K        [[F1]] = λxy.[[F]](xy) = B [[F]]

so every program is a pure `S`/`K`/`I`/`B` term — which our reducer reduces and
renders as ι directly. Every binary string is a valid program, and the binary
digits of an integer *n* (MSB first) are "program *n*" (the paper's Gödel
numbering). [`../../iota/jot.py`](../../iota/jot.py) does this translation:

```sh
python3 iota/jot.py catalog iota-examples/tarpit/jot-programs.txt   # -> dump lines
bash iota-examples/scripts/tarpit-film.sh DUMP ROOT out.mp4 STEPS SECS FPS comb
```

`jot-programs.txt` catalogs the paper's notable programs; `programs.dump` holds a
few hand-written divergent terms (`omega`, `grow`).

## The key finding (why we can't reproduce the paper's reductions *exactly*)

The paper's reductions are long — and sometimes non-terminating — because it
reduces **λ-terms to full normal form, reducing _under_ the abstractions**. Its
canonical divergent program reduces to `λx.(λyzx.zxz) x λyzx.zxz`, which only
loops because you keep reducing *inside* the `λ`.

Combinatory logic has **no abstractions**: an under-applied `S`/`K` is already a
value, so our reducer reaches *weak* normal form and stops. Translating the
paper's exact programs (incl. its applicative-order "infinite loops") and reducing
them here — even eagerly (`morph --ao`), even saturated with fresh variables —
they all **halt in ~30 steps**. The length is a *reduce-under-λ* artifact that
combinators structurally lack. Reproducing those exactly would need a
λ-calculus normal-form reducer and a λ-tree renderer (a different tool).

## What this directory *can* show as genuine long reductions

- **A real binary as a program** — `Jot.elf_true` is a 45-byte ELF `true`
  executable (360 bits) read as a Jot program. It starts as a ~8000-node ι
  mandala and reduces over hundreds of steps to its weak normal form.
- **Self-application divergence** — `omega = (λx.xx)(λx.xx)`,
  `grow = (λx.xxx)(λx.xxx)`: these diverge at the *application* level, so they are
  honestly infinite in the SK/ι tarpit (the mandala just grows; we cap at N steps).

`morph` takes `--ao` for applicative (eager, call-by-value) order vs the default
normal order — the paper is itself a study of reduction strategies (its Table 1).
