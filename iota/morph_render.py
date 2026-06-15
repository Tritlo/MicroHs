#!/usr/bin/env python3
"""Render an identity-tracked reduction trace as a smooth morphing animation.

Input (stdin): lines  RULE \t  (id label child...)   from iota/morph.
Output: numbered SVG frames in OUTDIR, tweening between consecutive terms by
matching node IDs (persisting nodes glide, new nodes grow in, dropped nodes
shrink out).  Variable timing keeps the whole thing under a target length.

  morph_render.py OUTDIR [target_seconds] [fps]
"""
import sys, os, math, re

BG="#0d1117"; FG="#e6edf3"; MUT="#6e7681"; ACC="#7ee787"
XS=24.0; YS=58.0; TOP=150.0
COMB=set("S K I B C A U Z P R O J Y".split()) | {"S'","B'","C'","C'B","K2","K3","K4"}

def color(label):
    if label=="@":     return MUT
    if label=="_|_":   return "#f85149"          # dead branch
    if label in COMB:  return "#58a6ff"          # combinator
    if label=="lt" or label=="test": return "#d2a8ff"   # function
    if label in ("T","F"): return "#7ee787"      # booleans
    return "#e3b341"                              # values / constructors

# ---------------------------------------------------------------- parse
def parse(s):
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
        label=toks[pos[0]]; pos[0]+=1
        kids=[]
        while toks[pos[0]]!=")": kids.append(go())
        pos[0]+=1
        return (nid,label,kids)
    return go()

# ---------------------------------------------------------------- layout (radial)
def nleaves(nd):
    return 1 if not nd[2] else sum(nleaves(k) for k in nd[2])

def layout(tree):
    # pos[id] = (angle, depth); leaves spread on the circle by in-order index
    pos={}; lab={}; kidsof={}; parent={}; lc=[0]; maxd=[0]
    nl=nleaves(tree)
    def walk(nd,d,par):
        nid,label,kids=nd
        lab[nid]=label; kidsof[nid]=[k[0] for k in kids]; parent[nid]=par
        maxd[0]=max(maxd[0],d)
        if not kids:
            ang=2*math.pi*lc[0]/max(1,nl); lc[0]+=1
        else:
            for k in kids: walk(k,d+1,nid)
            angs=[pos[k[0]][0] for k in kids]; ang=sum(angs)/len(angs)
        pos[nid]=(ang,d)
    walk(tree,0,None)
    return pos,lab,kidsof,parent,nl,maxd[0]

def hsl_hex(h,s,l):
    import colorsys
    r,g,b=colorsys.hls_to_rgb((h%360)/360.0,l,s)
    return "#%02x%02x%02x"%(int(r*255),int(g*255),int(b*255))

# ---------------------------------------------------------------- drawing
def ease(t): return t*t*(3-2*t)
def lerp(a,b,t): return a+(b-a)*t

def esc(s): return s.replace("&","&amp;").replace("<","&lt;").replace(">","&gt;")

RSTEP=22.0; CAPTOP=150; LEAF="#ffe08a"

def node_xy(pos,nid,cx,cy):
    ang,d=pos[nid]; r=d*RSTEP; return (cx+r*math.cos(ang), cy+r*math.sin(ang))

def compute_anchors(W,H,A,B):
    """For a transition A->B: where new nodes grow FROM (their nearest surviving
    ancestor) and where dropped nodes collapse INTO, so subtrees unfurl/retract
    from their attachment point instead of popping in/out of existence."""
    pA,parA=A[0],A[3]; pB,parB=B[0],B[3]
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    inA=set(pA); inB=set(pB)
    newsrc={}
    for nid in inB-inA:
        p=parB.get(nid)
        while p is not None and p not in inA: p=parB.get(p)
        newsrc[nid]=node_xy(pA,p,cx,cy) if p is not None else (cx,cy)
    dropdst={}
    for nid in inA-inB:
        p=parA.get(nid)
        while p is not None and p not in inB: p=parA.get(p)
        dropdst[nid]=node_xy(pB,p,cx,cy) if p is not None else (cx,cy)
    return (newsrc,dropdst)

def frame_svg(W,H, A,B, t, caption, term, expl, maxd, anchors=None):
    """radial morph; A,B = (pos,lab,kids,parent,nleaves,maxd); pos[id]=(angle,depth)."""
    pA,labA,kA,parA,nlA,mdA = A
    pB,labB,kB,parB,nlB,mdB = B
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    def px(pos,nid): return node_xy(pos,nid,cx,cy)
    te=ease(t)
    newsrc,dropdst = anchors if anchors else ({},{})
    out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" font-family="monospace">',
         f'<rect width="{W}" height="{H}" fill="{BG}"/>',
         f'<text x="{W/2}" y="50" font-size="34" fill="{FG}" text-anchor="middle">{esc(caption)}</text>']
    if term:
        out.append(f'<text x="{W/2}" y="90" font-size="26" fill="{MUT}" text-anchor="middle">{esc(term)}</text>')
    if expl:
        out.append(f'<text x="{W/2}" y="128" font-size="32" fill="{ACC}" text-anchor="middle">{esc(expl)}</text>')
    inA=set(pA); inB=set(pB)
    def cur(nid):                          # (x, y, alpha, depth)
        if nid in inB:
            tgt=px(pB,nid)
            if nid in inA: src=px(pA,nid); a=1.0
            else:          src=newsrc.get(nid,tgt); a=te      # grow out from attachment
            return (lerp(src[0],tgt[0],te), lerp(src[1],tgt[1],te), a, pB[nid][1])
        s=px(pA,nid); dst=dropdst.get(nid,s)                  # collapse into surviving ancestor
        return (lerp(s[0],dst[0],te), lerp(s[1],dst[1],te), 1-te, pA[nid][1])
    def edges(ids,kids):
        for nid in ids:
            x0,y0,a0,d0=cur(nid)
            col=hsl_hex(360.0*d0/max(1,maxd),0.85,0.62)
            for c in kids[nid]:
                x1,y1,a1,_=cur(c); a=min(a0,a1)*0.85
                if a>0.02:
                    out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="{col}" stroke-width="1" opacity="{a:.2f}"/>')
    edges(inB,kB); edges(inA-inB,kA)
    for nid in inB | (inA-inB):            # iota leaf dots only
        label = labB[nid] if nid in inB else labA[nid]
        if label!="1": continue
        x,y,a,_=cur(nid)
        if a>0.02:
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.4" fill="{LEAF}" opacity="{a:.2f}"/>')
    out.append("</svg>")
    return "\n".join(out)

# ---------------------------------------------------------------- main
def main():
    outdir=sys.argv[1]; target=float(sys.argv[2]) if len(sys.argv)>2 else 80.0
    fps=int(sys.argv[3]) if len(sys.argv)>3 else 30
    os.makedirs(outdir,exist_ok=True)
    steps=[]
    for line in sys.stdin:
        f=line.rstrip("\n").split("\t")
        if len(f)>=3:   rule,term,sx=f[0],f[1],f[2]
        elif len(f)==2: rule,term,sx=f[0],"",f[1]
        else:           rule,term,sx="","",f[0]
        steps.append((rule,term,parse(sx)))
    n=len(steps)
    lays=[layout(t) for _,_,t in steps]
    maxd=max(l[5] for l in lays)
    SZ=2*(maxd+1)*RSTEP
    W=int(SZ+120); H=int(CAPTOP+SZ+50)

    # The fixpoint step's rule carries the call it is unrolling, e.g.
    # "Y x -> x (Y x)   lt 2 3"; split that note off the caption and surface it.
    def caption_call(i):
        m=re.search(r"\s+lt (\d+) (\d+)\s*$", steps[i][0])
        if m: return steps[i][0][:m.start()], (int(m.group(1)), int(m.group(2)))
        return steps[i][0], None
    # the machine state (combinator term) shown on every frame, truncated
    def term_for(i):
        s=steps[i][1].strip()
        return s if len(s)<=72 else s[:69]+"..."
    # green insight on the recursive-call / result steps
    def expl_for(i):
        _,call=caption_call(i)
        if call: return f"lt {call[0]} {call[1]}      is {call[0]} < {call[1]} ?"
        if steps[i][1].strip()=="A": return "A  =  T  =  True"
        return ""

    # frame budget: a hold on each step + a tween into the next, fit to <= target s.
    # A faithful pure-combinator reduction is many (~150) substantial steps, so the
    # floors are small; "extra" frames are scaled to fill the budget and go to
    # (a) dwelling on the lt-call / result milestones and (b) smoother, longer morphs
    # on the big expansions (Y unrolls, S'/J duplications), where the most happens.
    total_frames=int(target*fps)
    sizes=[len(l[0]) for l in lays]
    trans=[abs(sizes[i+1]-sizes[i]) for i in range(n-1)]   # nodes added/removed i->i+1
    key=[bool(expl_for(i)) for i in range(n)]              # lt-call milestones + result
    isY=[steps[i][0].startswith("Y ") for i in range(n)]   # fixpoint unrolls (recursion)
    MIN_HOLD, MIN_TWEEN = 3, 5
    hold_extra =[(110.0 if i==n-1 else 55.0 if key[i] else 0.0) for i in range(n)]
    tween_extra=[(trans[i]**0.62) * (1.6 if isY[i+1] else 1.0) for i in range(n-1)]
    floor=MIN_HOLD*n + MIN_TWEEN*(n-1)
    raw=sum(hold_extra)+sum(tween_extra)
    scale=max(0.0, total_frames-floor)/raw if raw>0 else 0.0
    hold =[MIN_HOLD  + round(hold_extra[i]*scale)  for i in range(n)]
    tween=[MIN_TWEEN + round(tween_extra[i]*scale) for i in range(n-1)]
    fnum=0
    def write(svg):
        nonlocal fnum
        open(os.path.join(outdir,f"f{fnum:05d}.svg"),"w").write(svg); fnum+=1
    for i in range(n):
        cap = caption_call(i)[0]
        term=term_for(i); expl=expl_for(i)
        for _ in range(hold[i]):                     # hold current step
            write(frame_svg(W,H, lays[i],lays[i], 0.0, cap, term, expl, maxd))
        if i+1<n:
            cap2=caption_call(i+1)[0]; term2=term_for(i+1); expl2=expl_for(i+1)
            anch=compute_anchors(W,H,lays[i],lays[i+1])
            tw=tween[i]
            for f in range(tw):                      # morph to next
                t=(f+1)/tw; late=t>0.5
                write(frame_svg(W,H, lays[i],lays[i+1], t, cap2,
                                term2 if late else term, expl2 if late else expl, maxd, anch))
    print(fnum)

if __name__=="__main__":
    main()
