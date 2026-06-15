# Watching quicksort run, one ι-tree at a time

The film is a full run of **quicksort `[3, 1, 2]`** — written in ordinary Haskell,
compiled by MicroHs all the way down to combinators, then expanded to a single
combinator, **ι (iota)**. Every frame is the *whole program as it stands*, drawn as
a radial ι-tree (a "mandala"); each reduction step morphs one mandala into the next.
There are **no names in the tree** — it is pure ι throughout — so every step is a
genuine reduction, and the recursion is a real fixed-point combinator, not a name
lookup. A short **prelude** builds the input, the **sort** is the reduction itself,
and an **epilogue** reads the answer back out. A subtitle track (`quicksort_full.srt`)
narrates the ideas as it plays.

## TL;DR caption

> A real Haskell quicksort, evaluated at the level of a single combinator. Each
> mandala is the entire program as an iota tree; each morph is one reduction step.
> The recursion is the **Y fixed-point combinator**; the recurring musical phrase is
> a comparison ("is x ≤ the pivot?"). Watch it partition, expand from ~11,000 nodes
> to ~122,000, then collapse to the 1,977-node tree for the sorted list **[1, 2, 3]**.

## The program

```haskell
data Bool = F | T
data Nat  = Z | S Nat
data List = Nil | Cons Nat List

le :: Nat -> Nat -> Bool                       -- m <= n
leP, gtP :: Nat -> List -> List                -- keep elements <= p / > p
app :: List -> List -> List                    -- (++)

quicksort Nil         = Nil
quicksort (Cons p xs) = app (quicksort (leP p xs)) (Cons p (quicksort (gtP p xs)))

ex312 = quicksort (Cons 3 (Cons 1 (Cons 2 Nil)))   -- quicksort [3,1,2]
```

Pick a pivot `p`, partition the rest into `<= p` and `> p`, sort each side, and
concatenate. All in unary (Peano) arithmetic, with hand-rolled lists.

## From Haskell to a single combinator

MicroHs compiles this to **point-free combinator code**: no variables, just
combinators applied to each other. The classic basis is **S and K** (`K x y = x`,
`S x y z = x z (y z)`), which alone are Turing-complete. For compactness MicroHs
emits a richer set of **supercombinators** (`B C S' B' C' P R O U Z J A Y …`) — these
are the rule names you see firing in the top caption — but each one still reduces to
S and K. And S and K themselves collapse to one combinator, **ι**:

```
ι = λx. x S K        ιι = I        K = ι(ι(ιι))        S = ι(ι(ι(ιι)))
```

So the reducer rewrites supercombinators, but every frame is *drawn* as the
equivalent pure-ι tree: each **dot** is one ι, each **branch** an application. As a
prefix bit-code a term is `iota = 0 | 1 iota iota` (`0` is ι, `1` applies the next
two) — i.e. a binary tree.

The data comes along for free via **Scott encoding** — a value *is* the function that
picks a branch:

| constructor | combinator | meaning |
|---|---|---|
| `Z` (zero) | `K` | pick the zero branch |
| `S n` (successor) | `J` | pick the successor branch, hand over `n` |
| `Nil` | `K` | pick the empty branch |
| `Cons h t` | `O` | pick the cons branch with head `h`, tail `t` |
| `F` / `T` | `K` / `A` | the two boolean branches |

So `3 = S (S (S Z))` is the tower `J (J (J K))`, and `[3,1,2]` is `O 3 (O 1 (O 2 K))`.
There are no integers, booleans, or lists in the machine — only combinators, and
below them, only ι. The one *recursive* group (`quicksort`, `le`, `leP`, `gtP`,
`app`) is tied off with **Y**, so the closed term has nothing to "unfold": every
step is a real combinator rule.

## The three acts

**Prelude (~10 s).** The three input numbers appear as separate little ι-trees
(`3 1 2`, labelled), then are built into the list **cons by cons** (`3 1 [2]` →
`3 [1,2]` → `[3,1,2]`), and finally wrapped as `quicksort [3,1,2]` — which is exactly
the first frame of the reduction.

**The sort (~120 s, 428 steps).** `Y x → x (Y x)` unrolls the recursion on each call;
`S`/`S'`/`Y` duplicate subtrees (the tree **expands**, peaking around 122,000 nodes);
`K`/`A`/`J` select branches and discard the rest (it shrinks). Underneath it is the
quicksort recurrence — partition around the pivot, recurse, append — but no `leP`,
`gtP` or `app` *call* is visible; they are flattened into one combinator tree. The
run finally collapses to `O 1 (O 2 (O 3 K))` = **[1, 2, 3]** (1,977 nodes).

**Epilogue (~8 s).** The sorted list is held with a resolving chord, then **un-consed**
head-first (`[1,2,3]` → `1 [2,3]` → `1 2 [3]` → `1 2 3`) to reveal the three numbers
again — the mirror of the prelude.

## The soundtrack

The music is generated from the reduction (pure-stdlib synthesis) and locked to the
same frame schedule, so every note lands on its morph. In the default **`comb`** mode
**the combinator being applied picks the pitch** — each family shares a pitch class —
so recurring rules become recurring motifs: the `le` (≤) comparison is the phrase you
keep hearing. The `Y` fixpoint steps get a **low bass accent** (one per recursive
call), a soft **root drone** runs throughout, and a **sustained tonic chord** resolves
on the sorted result and again on the final reveal. (`MUSIC_MODE=size` instead maps
pitch to tree size, so the melody rises as the mandala expands.)

## Regenerating it

```sh
# 1. dump + reduce
bin/gmhs -iiota-examples/programs -ilib -ddump-combinator Quicksort | grep '^Quicksort\.' > /tmp/dump
iota/morph --iota /tmp/dump Quicksort.ex312 > /tmp/trace

# 2. prelude (numbers -> list -> quicksort term) and epilogue (un-cons the result)
iota/morph_intro.py /tmp/dump Quicksort.ex312 > /tmp/prelude.trace
tail -n 1 /tmp/trace | cut -f3 > /tmp/final.sx          # the real final term, for a seamless join
iota/morph_intro.py --outro --iotafile /tmp/final.sx --vals 1,2,3 > /tmp/uncons.trace

# 3. stitch:  prelude + reduction (steps 1..) + epilogue
cat /tmp/prelude.trace > /tmp/combined.trace
tail -n +2 /tmp/trace >> /tmp/combined.trace
cat /tmp/uncons.trace >> /tmp/combined.trace

# 4. render (target 124 s puts the sort itself at ~120 s; prelude/epilogue keep their pacing)
SEG_JOBS=3 MORPH_JOBS=6 iota/morph_film.py --trace /tmp/combined.trace iota-examples/films/morph_quicksort.mp4 124 30

# 5. (optional) burn in the subtitles
ffmpeg -i iota-examples/films/morph_quicksort.mp4 \
  -vf "subtitles=iota-examples/films/quicksort_full.srt:force_style='FontName=DejaVu Sans,Fontsize=12'" \
  -c:v libx264 -crf 20 -c:a copy iota-examples/films/morph_quicksort_explained.mp4
```

Pacing knobs live in the trace via `HOLD:` / `TWEEN:` hints (intro/outro frames) and a
`CHORD` marker (the held result); the renderer also honours `SEG_SECS`, `SEG_JOBS`,
`MORPH_JOBS`, `MORPH_MAXD`, and `MORPH_TWEEN_CAP`. See the combinator legend in
[EXPLAINER.md](EXPLAINER.md) for what each rule (`Y`, `S`, `K`, `J`, `O`, …) does.
