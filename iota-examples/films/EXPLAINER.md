# Watching `2 < 3` evaluate, one ι-tree at a time

The film is the reduction of a tiny Haskell program — `2 < 3` on unary
(Peano) numbers — compiled by MicroHs all the way down to combinators, then
expanded to **Barker's Iota** (`ι`, a single combinator). Every frame is the
*whole program as it stands*, drawn as a radial ι-tree (a "mandala"). Each
reduction step morphs one mandala into the next. There are **no names in the
tree** — it is pure ι throughout — so every step is a genuine reduction, and the
recursion is done by a real fixed-point combinator, not by looking a name up.

## TL;DR caption

> A one-line Haskell program, `2 < 3`, evaluated by hand at the level of pure
> combinators. Each mandala is the entire program as an iota tree; each morph is
> one reduction step. The recursion is the **Y fixed-point combinator** unrolling
> itself three times. Watch it peel the two numbers apart, throw away the branches
> it doesn't take, and collapse — from ~3,400 nodes to the 11-node tree for **True**.

## The program

```haskell
data Bool = F | T
data Nat  = Z | S Nat            -- 0, and "+1"

lt Z     (S _) = T               -- 0 < anything positive
lt _     Z     = F               -- nothing < 0
lt (S m) (S n) = lt m n          -- strip one S off each, recurse

test = lt (S (S Z)) (S (S (S Z)))   -- lt 2 3
```

MicroHs compiles this to **point-free combinator code**: no variables, just a
fixed alphabet of combinators (`S K B C S' C' J U R Z …`) applied to each other.
The data comes along for free via **Scott encoding** — a number/boolean *is* the
function that picks a branch:

- `0` is **K** (takes the first / zero branch)
- `S n` (successor) is **J n** (takes the second / successor branch, handing over the predecessor `n`)
- `True` is **A**, `False` is **K**

So a number is a tower: `2 = S (S Z)` becomes `J (J K)`, `3 = J (J (J K))`. There
are no integers or booleans in the machine at all — only combinators, and below
them, only ι.

## A closed term, and the one move that matters

To get a finite tree with no names, the whole program is turned into a **closed**
combinator term: every definition is inlined, and the one *recursive* definition
(`lt`) is tied off with the **Y combinator** — `lt = Y g`, where `g` is `lt`'s
body abstracted over its own name. `Y` is itself a closed combinator
(`B (S I I) (C B (S I I))`), so it too is just a region of ι in the tree.

Now there is nothing to "unfold." Every step is one of:

1. **A combinator rule** — a pure rewiring of the tree: copy a subtree, drop one,
   swap two. `K x y → x` (keep the first, bin the second), `S x y z → x z (y z)`
   (hand the same argument `z` to *both* sides — a duplication), `J x y z → z x`
   (a number revealing it's a successor).

2. **The fixpoint rule, `Y x → x (Y x)`** — this is the whole of recursion. Each
   time it fires, `Y g` becomes `g (Y g)`: the body `g` runs once more, with a
   fresh copy of `Y g` tucked inside for the *next* call. It fires exactly three
   times here — once per recursive call — and the film tags each with the call it
   is unrolling (`lt 2 3`, `lt 1 2`, `lt 0 1`).

## The walkthrough (what you're seeing)

The 82 steps are three rounds of recursion plus a base case:

**Round 1 — `lt 2 3`.** The term starts as `(Y g) 2 3`. The first step is
`Y x → x (Y x)`: the body unrolls to `g (Y g) 2 3`, carrying a copy of the
recursion inside. A run of `S / K / S' / C / C'` threads that copy into place and
lines the two numbers up against each other; the `S` / `S'` rules **duplicate the
continuation** (the spikes to ~10k nodes — comparing both successors needs two
copies). Then `J x y z → z x` fires the **successor branch** on each number,
stripping one `S` off both sides, and after the `K`s discard the branch not taken
it lands on `(Y g) 1 2`.

**Round 2 — `lt 1 2`.** The same shape, one size smaller: `Y` unrolls again,
duplicate, peel `1→0` and `2→1`, arrive at `(Y g) 0 1`.

**Round 3 — `lt 0 1`, the base case.** `Y` unrolls a third time, but now the left
number is `0` = **K**. So the abstraction selects the *zero-branch*: `K` keeps it
and discards the rest, `R x y z → y z x` rotates the constant `K A` into position,
and a final `J … (K A)` then `K A K` reduce to **A**. That 11-node tree is `True`.

**Why it balloons then collapses.** `Y` copies the body (the tree grows on every
unroll); `S` / `S'` duplicate arguments (the spikes); `K` / `J` select a branch
and **throw the other away** (the tree shrinks). The final cliff — thousands of
nodes down to 11 — is the answer crystallising out of all that scaffolding.

## Combinator legend (the rules that appear)

| rule | does |
|---|---|
| `Y x = x (Y x)` | **fixpoint** — the engine of recursion (here `Y = B (S I I) (C B (S I I))`) |
| `S x y z = x z (y z)` | share `z` with both sides (**duplicate**) |
| `S' x y z w = x (y w) (z w)` | `S`, carrying an extra arg `w` |
| `K x y = x` | keep first, **drop** second |
| `I x = x` | identity |
| `B x y z = x (y z)` | compose |
| `C x y z = x z y` | flip last two |
| `C' x y z w = x (y w) z` | flip, threading an extra arg |
| `C'B x y z w = x z (y w)` | a `C'`/`B` plumbing variant |
| `J x y z = z x` | **successor**: give predecessor `x` to the succ-branch `z` |
| `U x y = y x` | apply, flipped |
| `R x y z = y z x` | rotate three args |
| `Z x y z = x y` | drop the third arg |

The data constructors are inlined before reduction, so they never appear as
names: zero and successor are **K** and **J**, `True` is **A**. The only true atom
is `ι` itself — `K`, `S`, `J`, `Y`, every label above is a little ι-gadget, which
is why even `True` is a tree and not a point.

## The soundtrack

The music is generated from the reduction itself (pure-stdlib synthesis), locked
to the same frame schedule so every note lands on its morph. Each step plays a
note, and **the combinator being applied picks the pitch** (the default `comb`
mode) — so recurring rules become recurring motifs and you can *hear* the
S/K duplicate-and-drop rhythm and the `J` successor steps.

The mapping mirrors the maths: each combinator **family shares a pitch class**, and
a prime lifts it an octave — `S`=G, `S'`=G an octave up; `C`=D, `C'`=D an octave
up. `K`/`I` are the tonic (C), `J` the third (E). The climb is capped at one
octave so nothing turns shrill (the doubly-decorated `C'B` shares `C'`'s note, the
lone `R` shares `S'`'s), keeping the melody at or below G4 over a low bass. The
three **`Y` fixpoint steps get a bass accent** (you hear each recursive call), and
**`A = True` resolves on a sustained tonic chord**, held while the final frame
lingers, over a soft root drone. (An alternative `size` mode instead maps pitch to
the tree size, so the melody rises as the mandala expands and falls as it
collapses — `MUSIC_MODE=size`.)

It runs ~1:28, ending on `A = T = True`. The pace is rubato (big expansions
stretch, small reductions snap), averaging ~56 steps/min — roughly **112 BPM**
with a step on every half-note. Structural anchors for section changes: `lt 2 3`
≈ 0:02, `lt 1 2` ≈ 0:39, `lt 0 1` ≈ 1:15, `True` ≈ 1:27.
