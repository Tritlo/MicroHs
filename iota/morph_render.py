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

def _hex(c): return (int(c[1:3],16),int(c[3:5],16),int(c[5:7],16))
def mix(c1,c2,t):                          # blend two #rrggbb colours
    a,b=_hex(c1),_hex(c2)
    return "#%02x%02x%02x"%tuple(int(a[i]+(b[i]-a[i])*t) for i in range(3))

# ---------------------------------------------------------------- drawing
def ease(t): return t*t*(3-2*t)
def lerp(a,b,t): return a+(b-a)*t

# ---------------------------------------------------------------- timing (shared)
def caption_call(rule):
    """Split a fixpoint step's "Y x -> x (Y x)   lt 2 3" into (caption, (2,3))."""
    m=re.search(r"\s+lt (\d+) (\d+)\s*$", rule)
    if m: return rule[:m.start()], (int(m.group(1)), int(m.group(2)))
    return rule, None

def step_expl(step):                       # green insight on milestones
    _,call=caption_call(step[0])
    if call: return f"lt {call[0]} {call[1]}      is {call[0]} < {call[1]} ?"
    if step[1].strip()=="A": return "A  =  T  =  True"
    return ""

def schedule(steps, sizes, target, fps):
    """Frame budget shared by the renderer and the music so they stay in sync:
    a hold on each step + a tween into the next, fit to <= target seconds, with the
    extra frames going to the lt-call / result milestones and the big (Y / S')
    morphs.  Returns (hold[n], tween[n-1], starts[n])."""
    n=len(steps)
    trans=[abs(sizes[i+1]-sizes[i]) for i in range(n-1)]
    key=[bool(step_expl(steps[i])) for i in range(n)]
    isY=[steps[i][0].startswith("Y ") for i in range(n)]
    total_frames=int(target*fps); MIN_HOLD, MIN_TWEEN = 3, 5
    hold_extra =[(110.0 if i==n-1 else 55.0 if key[i] else 0.0) for i in range(n)]
    tween_extra=[(trans[i]**0.62) * (1.6 if isY[i+1] else 1.0) for i in range(n-1)]
    floor=MIN_HOLD*n + MIN_TWEEN*(n-1)
    raw=sum(hold_extra)+sum(tween_extra)
    sc=max(0.0, total_frames-floor)/raw if raw>0 else 0.0
    hold =[MIN_HOLD  + round(hold_extra[i]*sc) for i in range(n)]
    tween=[MIN_TWEEN + round(tween_extra[i]*sc) for i in range(n-1)]
    starts=[]; f=0
    for i in range(n):
        starts.append(f); f += hold[i] + (tween[i] if i+1<n else 0)
    return hold, tween, starts

def esc(s): return s.replace("&","&amp;").replace("<","&lt;").replace(">","&gt;")

RSTEP=22.0; CAPTOP=150; LEAF="#ffe08a"
GRAYC="#586069"          # a discarded branch (the road not taken)
COPYC="#ff9bd2"          # a freshly-copied argument splitting off its original
FLASH="#f0f6fc"          # the redex that just fired

def parse_prov(s):
    """4th trace field: 'R:id'(redex head) 'D:id'(discarded root) 'C:src:dst'(copy)."""
    redex=None; drop=[]; copy=[]
    for tok in s.split():
        if   tok.startswith("R:"): redex=int(tok[2:])
        elif tok.startswith("D:"): drop.append(int(tok[2:]))
        elif tok.startswith("C:"): a,b=tok[2:].split(":"); copy.append((int(a),int(b)))
    return {"redex":redex,"drop":drop,"copy":copy}

def node_xy(pos,nid,cx,cy):
    ang,d=pos[nid]; r=d*RSTEP; return (cx+r*math.cos(ang), cy+r*math.sin(ang))

def _subtree(root,kids):
    out=[]; st=[root]
    while st:
        n=st.pop()
        if n not in kids: continue
        out.append(n); st.extend(kids[n])
    return out

def _pair(src,dst,kids):
    """Pair the (isomorphic) copy subtree at dst with its original at src, both in
    the same tree -> {copyNode: origNode}."""
    m={}; st=[(src,dst)]
    while st:
        s,d=st.pop(); m[d]=s
        ks,kd=kids.get(s,[]),kids.get(d,[])
        if len(ks)==len(kd):
            for a,b in zip(ks,kd): st.append((a,b))
    return m

def compute_transition(W,H,A,B,prov):
    """Everything the tween A->B needs: attachment points for plain new/dropped
    nodes, plus the semantic roles from the reducer -- which A-nodes are a discarded
    branch (grey out), which B-nodes are a copy and of whom (fly out of the original),
    and the redex head that fired (flash)."""
    pA,kA,parA=A[0],A[2],A[3]; pB,kB,parB=B[0],B[2],B[3]
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    inA=set(pA); inB=set(pB)
    gray=set()
    for d in prov["drop"]:
        gray.update(_subtree(d,kA))
    copysrc={}; copyorig=set()
    for s,d in prov["copy"]:
        m=_pair(s,d,kB); copysrc.update(m); copyorig.update(m.values())
    newsrc={}
    for nid in inB-inA:
        if nid in copysrc: continue                       # copies fly from their original
        p=parB.get(nid)
        while p is not None and p not in inA: p=parB.get(p)
        newsrc[nid]=node_xy(pA,p,cx,cy) if p is not None else (cx,cy)
    return {"newsrc":newsrc,"gray":gray,"copysrc":copysrc,"copyorig":copyorig,
            "redex":prov.get("redex")}

def frame_svg(W,H, A,B, t, caption, term, expl, maxd, tr=None):
    """radial morph; A,B = (pos,lab,kids,parent,nleaves,maxd); pos[id]=(angle,depth)."""
    pA,labA,kA,parA,nlA,mdA = A
    pB,labB,kB,parB,nlB,mdB = B
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    def px(pos,nid): return node_xy(pos,nid,cx,cy)
    te=ease(t)
    tr=tr or {}
    newsrc=tr.get("newsrc",{}); gray=tr.get("gray",set())
    copysrc=tr.get("copysrc",{}); copyorig=tr.get("copyorig",set()); redex=tr.get("redex")
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
            if nid in copysrc:                                # a copy: fly out of its original
                o=copysrc[nid]; src=px(pA,o) if o in pA else px(pB,o); a=0.2+0.8*te
            elif nid in inA: src=px(pA,nid); a=1.0            # persists -> glide
            else:            src=newsrc.get(nid,tgt); a=te    # new machinery -> grow from attachment
            return (lerp(src[0],tgt[0],te), lerp(src[1],tgt[1],te), a, pB[nid][1])
        s=px(pA,nid)                                          # dropped -> fade out in place
        return (s[0], s[1], 1-te, pA[nid][1])
    def ncol(nid,d,inB_):                  # edge/dot colour by semantic role
        base=hsl_hex(360.0*d/max(1,maxd),0.85,0.62)
        if (not inB_) and nid in gray:        return GRAYC
        if nid in copysrc or nid in copyorig: return mix(COPYC, base, te)  # highlight, then settle
        return base
    def edges(ids,kids,inB_):
        for nid in ids:
            x0,y0,a0,_=cur(nid)
            for c in kids[nid]:
                x1,y1,a1,d1=cur(c); a=min(a0,a1)*0.85
                if a>0.02:
                    col=ncol(c,d1,inB_)
                    w=1.6 if (c in copysrc or c in copyorig) else 1.0
                    out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="{col}" stroke-width="{w}" opacity="{a:.2f}"/>')
    edges(inB,kB,True); edges(inA-inB,kA,False)
    for nid in inB | (inA-inB):            # iota leaf dots only
        inB_=nid in inB
        label = labB[nid] if inB_ else labA[nid]
        if label!="1": continue
        x,y,a,d=cur(nid)
        if a>0.02:
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="2.4" fill="{ncol(nid,d,inB_)}" opacity="{a:.2f}"/>')
    if redex is not None and redex in pA:  # flash where the rule fired
        rx,ry=px(pA,redex); fa=max(0.0,1.0-2.4*t)
        if fa>0.02:
            out.append(f'<circle cx="{rx:.1f}" cy="{ry:.1f}" r="{lerp(10,30,t):.1f}" fill="none" stroke="{FLASH}" stroke-width="2.5" opacity="{fa*0.85:.2f}"/>')
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
        rule = f[0] if f else ""
        if   len(f)>=3: term,sx,pv = f[1],f[2],(f[3] if len(f)>3 else "")
        elif len(f)==2: term,sx,pv = "",f[1],""
        else:           rule,term,sx,pv = "","",(f[0] if f else ""),""
        steps.append((rule,term,parse(sx),parse_prov(pv)))
    n=len(steps)
    lays=[layout(s[2]) for s in steps]
    maxd=max(l[5] for l in lays)
    SZ=2*(maxd+1)*RSTEP
    W=int(SZ+120); H=int(CAPTOP+SZ+50)

    # the machine state (combinator term) shown on every frame, truncated
    def term_for(i):
        s=steps[i][1].strip()
        return s if len(s)<=72 else s[:69]+"..."
    def expl_for(i): return step_expl(steps[i])

    sizes=[len(l[0]) for l in lays]
    hold, tween, _ = schedule(steps, sizes, target, fps)
    fnum=0
    def write(svg):
        nonlocal fnum
        open(os.path.join(outdir,f"f{fnum:05d}.svg"),"w").write(svg); fnum+=1
    for i in range(n):
        cap = caption_call(steps[i][0])[0]
        term=term_for(i); expl=expl_for(i)
        for _ in range(hold[i]):                     # hold current step
            write(frame_svg(W,H, lays[i],lays[i], 0.0, cap, term, expl, maxd))
        if i+1<n:
            cap2=caption_call(steps[i+1][0])[0]; term2=term_for(i+1); expl2=expl_for(i+1)
            tr=compute_transition(W,H,lays[i],lays[i+1], steps[i+1][3])   # roles for i->i+1
            tw=tween[i]
            for f in range(tw):                      # morph to next
                t=(f+1)/tw; late=t>0.5               # caption/term/expl all track the
                write(frame_svg(W,H, lays[i],lays[i+1], t,   # currently-dominant step,
                                cap2 if late else cap,       # so we show the ongoing
                                term2 if late else term,     # reduction, not the next one
                                expl2 if late else expl, maxd, tr))
    print(fnum)

if __name__=="__main__":
    main()
