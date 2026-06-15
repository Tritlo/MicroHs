#!/usr/bin/env python3
"""Translate Jot / Iota programs into combinator dumps for iota/morph.

From Hemann & Holk, "Visualizing the Turing Tarpit".  A Jot program is a binary
string with semantics

    [[ε]]  = I            [[F0]] = [[F]] S K            [[F1]] = λxy.[[F]](xy) = B [[F]]

so a program is a pure S/K/I/B term.  Every binary string is a valid program, and
the binary digits of an integer n (MSB first) are "program n" -- the paper's
Gödel numbering.  Hence a program can be named either way:

    jot.py int 556550 [name]     # by Gödel number
    jot.py bits 01101 [name]     # by binary string
    jot.py catalog FILE          # emit a dump from a catalog of  name<space>int:N|bits:...

Output is dump lines  `name = <term>`  ready for iota/morph (feed via tarpit-film.sh).
"""
import sys

def jot_to_sk(bits):
    """Jot binary string (MSB first) -> S/K/I/B combinator term string."""
    t = "I"
    for b in bits:
        if   b == "0": t = f"({t} S K)"    # [[F0]] = [[F]] S K
        elif b == "1": t = f"(B {t})"      # [[F1]] = B [[F]]
        else: raise ValueError(f"not a bit: {b!r}")
    return t

def bits_of(spec):
    if spec.startswith("int:"):  return bin(int(spec[4:]))[2:]
    if spec.startswith("bits:"): return spec[5:].replace(" ", "")
    raise ValueError(f"expected int:N or bits:..., got {spec!r}")

def main(argv):
    if len(argv) >= 2 and argv[0] == "catalog":
        for line in open(argv[1]):
            line = line.split("#", 1)[0].strip()
            if not line: continue
            name, spec = line.split(None, 1)
            print(f"{name} = {jot_to_sk(bits_of(spec.strip()))}")
        return
    if len(argv) >= 2 and argv[0] in ("int", "bits"):
        bits = bits_of(f"{argv[0]}:{argv[1]}")
        name = argv[2] if len(argv) > 2 else "Jot.prog"
        print(f"{name} = {jot_to_sk(bits)}")
        return
    sys.exit(__doc__)

if __name__ == "__main__":
    main(sys.argv[1:])
