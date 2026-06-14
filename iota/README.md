# iota — render MicroHs programs as Barker-Iota binary trees

Take a MicroHs program, extract the combinators it compiles to, translate those
to [Barker's Iota](https://en.wikipedia.org/wiki/Iota_and_Jot)
(`iota = "1" | "0" iota iota`, where `1` is the combinator `ι = λf.f S K` and
`0 a b` applies `a` to `b`), and render the result as an ASCII or PNG binary tree.

## Pipeline

```
MicroHs source ──gmhs -ddump-combinator──► combinator term (S K I B C S' … Y)
               ──bracket-abstract each combinator's rule──► pure S/K
               ──S → 010101011, K → 0101011, app → 0──────► iota 0/1 string
               ──layout──────────────────────────────────► ASCII / SVG / PNG
```

Only *closed, pure-combinatory* terms get a real iota string. Runtime
primitives (arithmetic `+`, `IO.*`, machine literals, FFI) have no combinatory
form, so they render as opaque leaf **boxes** (`<+>`, `<IO.return>`, …) and the
iota encoding is suppressed for that term.

## Build

```sh
ghc -O0 -o iota/iota  iota/Iota.hs     # the renderer
ghc -O0 -o iota/check iota/Check.hs    # the validating reducer
```

## Usage

Get a dump (note: use `gmhs`, the GHC-built compiler; the prebuilt `bin/mhs`
errors here). A module with `import qualified Prelude()` and no `main` keeps the
term primitive-free:

```sh
gmhs -i. -ilib -ddump-combinator MyModule 2>/dev/null | grep ' = ' > my.dump
```

```sh
iota my.dump  Mod.name [maxIotaTreeNodes]   # ASCII trees + iota string (default)
iota sexp  my.dump Mod.name                 # combinator tree as an s-expression
iota sk    my.dump Mod.name                 # the pure S/K term
iota iota  my.dump Mod.name                 # just the 0/1 iota string
```

PNGs (needs `python3` and ImageMagick `convert`; no graphviz required):

```sh
iota sexp my.dump Mod.name | python3 iota/treedraw.py topdown sexp out.svg "title"
iota iota my.dump Mod.name | python3 iota/treedraw.py radial  iota out.svg "title"
convert -density 140 out.svg out.png
```

`topdown` is a tidy labelled layout (good for small combinator trees);
`radial` places the root at the centre with edges coloured by depth (legible for
the huge iota expansions).

## Validation

`check` parses a 0/1 iota string, applies it to free argument names, and
normal-order reduces (`ι x = x S K`) so you can confirm meaning is preserved:

```sh
iota iota my.dump Mod.six | xargs -I{} ./iota/check {} g y
#  →  (g (g (g (g (g (g y))))))     -- six applies its argument 6 times
```
