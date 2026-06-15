#!/usr/bin/env python3
"""Lay out a binary tree as SVG.

Inputs (read from stdin):
  - an s-expression: atoms are leaf labels, (L R) is a binary node; or
  - a Barker-Iota 0/1 string: '1' is a leaf, '0' a b is a node.

Modes:
  topdown  tidy top-down layout with labelled nodes (good for small trees)
  radial   root-centred radial layout, edges coloured by depth (huge trees)

Usage:  treedraw.py {topdown|radial} {sexp|iota} OUT.svg [TITLE]
"""
import sys, os

# radial colours, overridable via env so the render script can drive colorschemes
_BG = os.environ.get("IOTA_BG", "#0b0f17")        # canvas
_LEAF = os.environ.get("IOTA_LEAF", "#ffe08a")    # iota leaf dots
_EDGE_L = float(os.environ.get("IOTA_EDGE_L", "0.60"))  # edge HSL lightness

# ----------------------------------------------------------------- parsing
class Node:
    __slots__ = ("label", "l", "r", "x", "y", "depth", "leaves")
    def __init__(self, label=None, l=None, r=None):
        self.label, self.l, self.r = label, l, r
        self.x = self.y = self.depth = 0.0
        self.leaves = 0
    def is_leaf(self):
        return self.l is None

def parse_sexp(s):
    toks, i, n = [], 0, len(s)
    while i < n:
        c = s[i]
        if c in "() ":
            if c != " ":
                toks.append(c)
            i += 1
        else:
            j = i
            while j < n and s[j] not in "() ":
                j += 1
            toks.append(s[i:j]); i = j
    pos = [0]
    def go():
        t = toks[pos[0]]; pos[0] += 1
        if t == "(":
            a = go(); b = go()
            assert toks[pos[0]] == ")"; pos[0] += 1
            return Node(label="@", l=a, r=b)
        return Node(label=t)
    return go()

def parse_iota(s):
    pos = [0]
    def go():
        c = s[pos[0]]; pos[0] += 1
        if c == "0":
            return Node(label="@", l=go(), r=go())
        return Node(label="1")
    return go()

# ------------------------------------------------------------- measurement
def annotate(root):
    """post-order: depth, leaf count, in-order leaf index -> x; returns #leaves, maxdepth."""
    leaf_counter = [0]
    maxdepth = [0]
    def walk(nd, d):
        nd.depth = d
        maxdepth[0] = max(maxdepth[0], d)
        if nd.is_leaf():
            nd.x = leaf_counter[0]
            leaf_counter[0] += 1
            nd.leaves = 1
            return
        walk(nd.l, d + 1); walk(nd.r, d + 1)
        nd.x = 0.5 * (nd.l.x + nd.r.x)
        nd.leaves = nd.l.leaves + nd.r.leaves
    walk(root, 0)
    return leaf_counter[0], maxdepth[0]

def each(root):
    st = [root]
    while st:
        nd = st.pop()
        yield nd
        if not nd.is_leaf():
            st.append(nd.l); st.append(nd.r)

# ----------------------------------------------------------------- styling
def leaf_style(label):
    """returns (shape, fill, textfill) for a leaf label."""
    if label == "1":                       # iota combinator
        return ("dot", "#111", None)
    if label.startswith("<"):              # primitive / literal box
        return ("box", "#e8743b", "#fff")
    return ("circle", "#3b78e8", "#fff")   # structural combinator

# --------------------------------------------------------------- top-down
def svg_topdown(root, title):
    nleaves, maxd = annotate(root)
    XS, YS, PAD = 56.0, 64.0, 40.0
    W = max(1, nleaves) * XS + 2 * PAD
    H = (maxd + 1) * YS + 2 * PAD + 24
    def px(nd): return PAD + nd.x * XS + XS / 2
    def py(nd): return PAD + 24 + nd.depth * YS
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W:.0f}" height="{H:.0f}" '
           f'viewBox="0 0 {W:.0f} {H:.0f}" font-family="monospace">']
    out.append(f'<rect width="{W:.0f}" height="{H:.0f}" fill="white"/>')
    if title:
        out.append(f'<text x="{PAD}" y="22" font-size="16" fill="#333">{esc(title)}</text>')
    for nd in each(root):                  # edges first
        if not nd.is_leaf():
            for c in (nd.l, nd.r):
                out.append(f'<line x1="{px(nd):.1f}" y1="{py(nd):.1f}" '
                           f'x2="{px(c):.1f}" y2="{py(c):.1f}" stroke="#aaa" stroke-width="1.5"/>')
    for nd in each(root):                  # nodes on top
        x, y = px(nd), py(nd)
        if nd.is_leaf():
            shape, fill, tf = leaf_style(nd.label)
            if shape == "box":
                w = 12 + 8 * len(nd.label)
                out.append(f'<rect x="{x-w/2:.1f}" y="{y-13:.1f}" width="{w:.1f}" height="26" rx="4" '
                           f'fill="{fill}"/>')
                out.append(f'<text x="{x:.1f}" y="{y+5:.1f}" font-size="14" text-anchor="middle" '
                           f'fill="{tf}">{esc(nd.label)}</text>')
            elif shape == "dot":
                out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="5" fill="{fill}"/>')
            else:
                out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="15" fill="{fill}"/>')
                out.append(f'<text x="{x:.1f}" y="{y+5:.1f}" font-size="15" text-anchor="middle" '
                           f'fill="{tf}">{esc(nd.label)}</text>')
        else:
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="6" fill="#888"/>')  # application
    out.append("</svg>")
    return "\n".join(out)

# ----------------------------------------------------------------- radial
def svg_radial(root, title):
    import math
    nleaves, maxd = annotate(root)
    # angle: leaves spread on circle; internal = midpoint of children. radius ~ depth.
    leaf_i = [0]
    def setang(nd):
        if nd.is_leaf():
            nd.y = 2 * math.pi * leaf_i[0] / max(1, nleaves)  # reuse .y as angle
            leaf_i[0] += 1
            return
        setang(nd.l); setang(nd.r)
        nd.y = 0.5 * (nd.l.y + nd.r.y)
    setang(root)
    R = 26.0
    SZ = 2 * (maxd + 1) * R + 80
    cx = cy = SZ / 2
    def pos(nd):
        rr = nd.depth * R
        return cx + rr * math.cos(nd.y), cy + rr * math.sin(nd.y)
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{SZ:.0f}" height="{SZ:.0f}" '
           f'viewBox="0 0 {SZ:.0f} {SZ:.0f}" font-family="monospace">']
    out.append(f'<rect width="{SZ:.0f}" height="{SZ:.0f}" fill="{_BG}"/>')
    for nd in each(root):
        if not nd.is_leaf():
            x0, y0 = pos(nd)
            col = hsl_hex(360.0 * nd.depth / max(1, maxd), 0.85, _EDGE_L)
            for c in (nd.l, nd.r):
                x1, y1 = pos(c)
                out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                           f'stroke="{col}" stroke-width="1" opacity="0.85"/>')
    for nd in each(root):
        if nd.is_leaf():
            x, y = pos(nd)
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.2" fill="{_LEAF}"/>')
    if title:
        out.append(f'<text x="16" y="28" font-size="16" fill="#cdd6e6">{esc(title)}</text>')
    out.append("</svg>")
    return "\n".join(out)

def hsl_hex(h, s, l):
    import colorsys
    r, g, b = colorsys.hls_to_rgb((h % 360) / 360.0, l, s)
    return "#%02x%02x%02x" % (int(r * 255), int(g * 255), int(b * 255))

def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")

# -------------------------------------------------------------------- main
def main():
    layout, kind, out = sys.argv[1], sys.argv[2], sys.argv[3]
    title = sys.argv[4] if len(sys.argv) > 4 else ""
    data = sys.stdin.read().strip()
    root = parse_iota(data) if kind == "iota" else parse_sexp(data)
    svg = svg_radial(root, title) if layout == "radial" else svg_topdown(root, title)
    with open(out, "w") as f:
        f.write(svg)
    n = sum(1 for _ in each(root))
    print(f"wrote {out}  ({n} nodes)")

if __name__ == "__main__":
    main()
