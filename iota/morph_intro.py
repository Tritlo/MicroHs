#!/usr/bin/env python3
"""Generate a short "intro" morph trace for a list program: show the input numbers,
then the input list, then the full (function applied to the list) term -- each as an
iota tree.  Because every frame is pruned out of the real first term's iota s-expr,
the shared nodes keep their ids and morph smoothly into the reduction that follows.

  morph_intro.py DUMP ROOT            # intro: ROOT = `func (Cons .. Nil)`, e.g. Quicksort.ex312
  morph_intro.py --outro DUMP ROOT    # outro: ROOT is already a list (the sorted result)

Prints trace lines  RULE \\t TERM \\t SX \\t PROV  to stdout.  Intro frames (numbers ->
list -> full term) prepend to ROOT's reduction trace; outro frames (list -> numbers,
the reveal, ending on the resolving chord) append to it.  Assumes Scott-encoded naturals
(Cons=O, Nil/Z=K, S=J), as MicroHs produces."""
import sys, subprocess, os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MORPH = os.path.join(ROOT, "iota", "morph")

def parse_sx(s):                       # (id label child...) -> [id, label, [kids]]
    toks=[]; i=0; n=len(s)
    while i<n:
        c=s[i]
        if c in "()": toks.append(c); i+=1
        elif c==" ": i+=1
        else:
            j=i
            while j<n and s[j] not in "() ": j+=1
            toks.append(s[i:j]); i=j
    pos=[0]
    def go():
        assert toks[pos[0]]=="("; pos[0]+=1
        nid=int(toks[pos[0]]); pos[0]+=1
        lab=toks[pos[0]]; pos[0]+=1
        kids=[]
        while toks[pos[0]]!=")": kids.append(go())
        pos[0]+=1
        return [nid,lab,kids]
    return go()

def ser(t):                            # back to the (id label child...) text form
    nid,lab,kids=t
    if not kids: return f"({nid} {lab})"
    return f"({nid} {lab} " + " ".join(ser(k) for k in kids) + ")"

def find(t, target):                   # first node with this id (ids are unique per term)
    if t[0]==target: return t
    for k in t[2]:
        r=find(k, target)
        if r: return r
    return None

def maxabs(t):
    return max(abs(t[0]), max((maxabs(k) for k in t[2]), default=0))

def morph_line(args):
    out = subprocess.run([MORPH]+args, capture_output=True, text=True, check=True).stdout
    return out.splitlines()[0]

def main():
    args=sys.argv[1:]
    outro = "--outro" in args
    def popflag(name):
        if name in args: i=args.index(name); v=args[i+1]; del args[i:i+2]; return v
        return None
    iotafile = popflag("--iotafile")   # build outro frames from a given final-term iota SX (real reduction ids)
    valstr   = popflag("--vals")        # the list-order values for that term (e.g. the sorted result 1,2,3)
    pos=[a for a in args if a!="--outro"]

    if iotafile:                       # outro joined onto the reduction: walk the cons spine in iota directly
        iota = parse_sx(open(iotafile).read().strip())
        vin=[int(x) for x in valstr.split(",")]
        cells=[]; cur=iota              # cons cell = `@ (@ Ogadget num) tail`; Nil's gadget has a leaf left
        while cur[1]=="@" and cur[2][0][1]=="@":
            cells.append((cur[0], cur[2][0][2][1][0], None, cur[2][1][0])); cur=cur[2][1]
        if len(cells)!=len(vin): sys.exit(f"morph_intro: {len(cells)} cells vs {len(vin)} values")
        cells=[(c[0],c[1],vin[i],c[3]) for i,c in enumerate(cells)]
    else:
        dump, root = pos[0], pos[1]
        comb = parse_sx(morph_line([dump, root, "0"]).split("\t")[1])          # combinator level: O/J/K legible
        iota = parse_sx(morph_line(["--iota", dump, root, "0"]).split("\t")[2])# iota level: what the film renders
        # walk the Cons spine `@ (@ O num) tail`: ROOT is `func list` (intro) or the list (outro).
        def is_cons(nd): return nd[1]=="@" and nd[2][0][1]=="@" and nd[2][0][2][0][1]=="O"
        listnode = comb if is_cons(comb) else comb[2][1]
        cells=[]; cur=listnode          # (cell_id, num_id, value, tail_id) in list order
        while is_cons(cur):
            numnode=cur[2][0][2][1]; tail=cur[2][1]
            v=0; t=numnode
            while t[1]=="@" and t[2][0][1]=="J": v+=1; t=t[2][1]
            cells.append((cur[0], numnode[0], v, tail[0])); cur=tail
    if not cells:
        sys.exit("morph_intro: found no Cons/number spine in ROOT")
    m=len(cells)
    nums   =[c[1] for c in cells]                          # number ids, in list order
    vals   =[c[2] for c in cells]                          # their values
    suffix =[c[0] for c in cells]                          # suffix[j] = the list from position j onward

    sid=[maxabs(iota)+1000]
    def mkrow(col_ids):                                    # synthetic ROW root over these subtree columns
        sid[0]+=1; return [sid[0],"ROW",[find(iota,c) for c in col_ids]]
    def lbls(vs):  return "ROWLBL:"+"|".join(vs)           # '|' separated: labels themselves contain commas
    def emit(rule, term, tree, prov): sys.stdout.write(f"{rule}\t{term}\t{ser(tree)}\t{prov}\n")
    def numlbl(i): return str(cells[i][2])                 # a number column's label
    def listlbl(j): return "["+",".join(str(cells[i][2]) for i in range(j,m))+"]"  # the suffix list from j
    listtxt=", ".join(map(str,vals))                       # in list order (no reordering)

    if outro:                          # sorted list (held, with a chord) -> uncons head-first -> the numbers
        # the full list, radial + full-scale (matches the reduction's last frame -> zero-morph join), rung
        emit(f"sorted!  [{listtxt}]", "", find(iota,suffix[0]), "HOLD:2.5 CHORD")
        for j in range(1, m):          # columns: first j numbers peeled off + the remaining tail list
            cols=[cells[i][1] for i in range(j)]+[suffix[j]]
            lab =[numlbl(i) for i in range(j)]+[listlbl(j)]
            emit("uncons", "", mkrow(cols), lbls(lab)+" HOLD:1.0")
        emit("...which is", "", mkrow(nums), lbls([numlbl(i) for i in range(m)])+" HOLD:2.4")
    else:                              # numbers (input order) -> build via cons -> quicksort applied
        # TWEEN slows each prelude morph (the global schedule otherwise snaps them, since the sort dominates)
        emit("the numbers", "", mkrow(nums), lbls([numlbl(i) for i in range(m)])+" HOLD:2.2 TWEEN:0.7")
        for j in range(m-1, 0, -1):    # columns: first j numbers loose + the suffix list they cons into
            cols=[cells[i][1] for i in range(j)]+[suffix[j]]
            lab =[numlbl(i) for i in range(j)]+[listlbl(j)]
            emit("build via cons" if j==m-1 else "cons", "", mkrow(cols), lbls(lab)+" HOLD:1.2 TWEEN:0.7")
        emit("the input list", "", mkrow([suffix[0]]), lbls([listlbl(0)])+" HOLD:1.4 TWEEN:0.7")
        emit(f"quicksort [{listtxt}]", "the reduction begins", iota, "HOLD:1.4")

if __name__=="__main__":
    main()
