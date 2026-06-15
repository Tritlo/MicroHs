#!/usr/bin/env python3
"""Render an identity-tracked reduction trace as a smooth morphing animation.

Input (stdin): lines  RULE \t  (id label child...)   from iota/morph.
Output: numbered SVG frames in OUTDIR, tweening between consecutive terms by
matching node IDs (persisting nodes glide, new nodes grow in, dropped nodes
shrink out).  Variable timing keeps the whole thing under a target length.

  morph_render.py OUTDIR [target_seconds] [fps]
"""
import sys, os, math

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

RSTEP=22.0; CAPTOP=126; LEAF="#ffe08a"

def frame_svg(W,H, A,B, t, caption, expl, maxd):
    """radial morph; A,B = (pos,lab,kids,parent,nleaves,maxd); pos[id]=(angle,depth)."""
    pA,labA,kA,parA,nlA,mdA = A
    pB,labB,kB,parB,nlB,mdB = B
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    def px(pos,nid):
        ang,d=pos[nid]; r=d*RSTEP; return (cx+r*math.cos(ang), cy+r*math.sin(ang))
    te=ease(t)
    out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" font-family="monospace">',
         f'<rect width="{W}" height="{H}" fill="{BG}"/>',
         f'<text x="{W/2}" y="56" font-size="36" fill="{FG}" text-anchor="middle">{esc(caption)}</text>']
    if expl:
        out.append(f'<text x="{W/2}" y="104" font-size="32" fill="{ACC}" text-anchor="middle">{esc(expl)}</text>')
    inA=set(pA); inB=set(pB)
    def cur(nid):                          # (x, y, alpha, depth)
        if nid in inB:
            tgt=px(pB,nid); src=px(pA,nid) if nid in inA else tgt
            a=1.0 if nid in inA else te
            return (lerp(src[0],tgt[0],te), lerp(src[1],tgt[1],te), a, pB[nid][1])
        s=px(pA,nid); return (s[0],s[1],1-te, pA[nid][1])
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

    # explanations on the recursive-call / result steps (from the combinator term)
    def expl_for(i):
        s=steps[i][1].strip(); parts=s.split()
        if len(parts)==3 and parts[0]=="lt": return f"is {parts[1]} < {parts[2]} ?"
        if s=="A": return "A  =  T  =  True"
        return ""

    # frame budget: hold + tween per step, scaled to ~target seconds
    total_frames=int(target*fps)
    # weight: expansions (big node delta) and key steps get more time
    import_w=[]
    for i in range(n):
        delta = abs(len(lays[i][0]) - (len(lays[i-1][0]) if i>0 else 0))
        key = 1 if expl_for(i) else 0
        import_w.append(1 + 0.04*delta + 1.4*key)
    wsum=sum(import_w)
    fnum=0
    def write(svg):
        nonlocal fnum
        open(os.path.join(outdir,f"f{fnum:05d}.svg"),"w").write(svg); fnum+=1
    for i in range(n):
        rule,_,_=steps[i]
        budget=max(6, round(total_frames*import_w[i]/wsum))
        hold=max(3, round(budget*0.45)); tween=budget-hold if i+1<n else 0
        cap = rule if rule else ""
        expl=expl_for(i)
        for _ in range(hold):                       # hold current step
            write(frame_svg(W,H, lays[i],lays[i], 0.0, cap, expl, maxd))
        if i+1<n:
            rule2,_,_=steps[i+1]; expl2=expl_for(i+1)
            for f in range(tween):                  # morph to next
                t=(f+1)/tween
                write(frame_svg(W,H, lays[i],lays[i+1], t, rule2, expl2 if t>0.5 else expl, maxd))
    print(fnum)

if __name__=="__main__":
    main()
