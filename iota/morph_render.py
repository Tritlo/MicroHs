#!/usr/bin/env python3
"""Render an identity-tracked reduction trace as a smooth morphing animation.

Input (stdin): lines  RULE \t  (id label child...)   from iota/morph.
Output: numbered SVG frames in OUTDIR, tweening between consecutive terms by
matching node IDs (persisting nodes glide, new nodes grow in, dropped nodes
shrink out).  Variable timing keeps the whole thing under a target length.

  morph_render.py OUTDIR [target_seconds] [fps]
"""
import sys, os, math, re
sys.setrecursionlimit(1_000_000)   # deep left-nested programs (e.g. a binary read as Jot) nest far past the default

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

def scan_sx(s):
    """Node count and max node-depth of a parenthesised term WITHOUT building the tree.
    Every node is one `( ... )` (labels/ids never contain parens), so the open-paren
    count is the node count and the deepest nesting is the depth.  Node depth counts
    the root as 0 (matching layout()), i.e. one less than the paren nesting.  Lets the
    --dims pass and per-segment sizing skip parsing the (huge) terms entirely."""
    nodes=0; depth=0; mx=0
    for c in s:
        if c=="(":
            nodes+=1; depth+=1
            if depth>mx: mx=depth
        elif c==")": depth-=1
    return nodes, max(0, mx-1)

# ---------------------------------------------------------------- layout (radial)
def nleaves(nd):
    return 1 if not nd[2] else sum(nleaves(k) for k in nd[2])

def layout_row(tree):
    """A synthetic 'ROW' root: lay each child subtree out radially in its OWN column
    (positions tagged ('R', col, ncols, angle, depth)).  The root and its edges are not
    drawn, so the children read as a left-to-right row of separate iota trees -- used for
    the intro frame that shows the input numbers before they combine into a list."""
    nid0,_,kids=tree
    pos={nid0:(0.0,0)}; lab={nid0:"ROW"}; kidsof={nid0:[k[0] for k in kids]}; parent={nid0:None}
    k=len(kids); mx=0
    for col,child in enumerate(kids):
        cpos,clab,ckids,cpar,_,cmd=layout(child); mx=max(mx,cmd)
        for cid,(a,d) in cpos.items(): pos[cid]=('R',col,k,a,d)
        lab.update(clab); kidsof.update(ckids); cpar[child[0]]=nid0; parent.update(cpar)
    return pos,lab,kidsof,parent,k,mx

def layout(tree):
    if tree[1]=="ROW": return layout_row(tree)
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

import functools
@functools.lru_cache(maxsize=4096)
def _hex(c): return (int(c[1:3],16),int(c[3:5],16),int(c[5:7],16))
@functools.lru_cache(maxsize=8192)
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

def schedule(steps, sizes, target, fps, hold_hint=None, tween_hint=None):
    """Frame budget shared by the renderer and the music so they stay in sync:
    a hold on each step + a tween into the next, fit to <= target seconds, with the
    extra frames going to the lt-call / result milestones and the big (Y / S')
    morphs.  hold_hint[i] / tween_hint[i] (seconds, optional) force a minimum hold on
    step i / morph out of step i -- used by intro/outro frames that must linger or morph
    slowly enough to read.  Returns (hold, tween, starts)."""
    n=len(steps)
    trans=[abs(sizes[i+1]-sizes[i]) for i in range(n-1)]
    key=[bool(step_expl(steps[i])) for i in range(n)]
    def is_copy(rule):                                  # S / S' / Y duplicate an argument
        h=rule.split(); return bool(h) and h[0] in ("S","S'","Y")
    total_frames=int(target*fps); MIN_HOLD, MIN_TWEEN = 3, 5
    hold_extra =[(110.0 if i==n-1 else 55.0 if key[i] else 0.0) for i in range(n)]
    # copying morphs (esp. the big ones) get more frames so the duplication reads;
    # MORPH_TWEEN_CAP clamps a giant size-jump morph (e.g. list -> full term) so it
    # doesn't hog the timeline.
    cap=float(os.environ.get("MORPH_TWEEN_CAP","1e18"))
    tween_extra=[min(cap,(trans[i]**0.62) * (1.9 if is_copy(steps[i+1][0]) else 1.0)) for i in range(n-1)]
    floor=MIN_HOLD*n + MIN_TWEEN*(n-1)
    raw=sum(hold_extra)+sum(tween_extra)
    sc=max(0.0, total_frames-floor)/raw if raw>0 else 0.0
    hold =[MIN_HOLD  + round(hold_extra[i]*sc) for i in range(n)]
    if hold_hint:                                       # guarantee a readable minimum on flagged frames
        hold=[max(hold[i], round(hold_hint[i]*fps)) for i in range(n)]
    tween=[MIN_TWEEN + round(tween_extra[i]*sc) for i in range(n-1)]
    if tween_hint:                                      # guarantee a minimum morph length on flagged frames
        tween=[max(tween[i], round(tween_hint[i]*fps)) for i in range(n-1)]
    starts=[]; f=0
    for i in range(n):
        starts.append(f); f += hold[i] + (tween[i] if i+1<n else 0)
    return hold, tween, starts

def esc(s): return s.replace("&","&amp;").replace("<","&lt;").replace(">","&gt;")

RSTEP=22.0; CAPTOP=150; LEAF="#ffe08a"
GRAYC="#586069"          # a discarded branch (the road not taken)
CPFLASH="#ff9bd2"        # the copy's spine flashes this, then emerges into normal colour
FLASH="#dfe7ef"          # twinkling sparkle where the rule fired
DRIFT=18.0               # how far a discarded branch drifts outward as it fades
TRAILC="#9aa7b3"; TRAIL_THRESH=26.0   # faint trail showing a rearranged argument's path
ROW_SCALE=0.55           # intro "row" frames: each number's iota gadget drawn this fraction of full size

def parse_prov(s):
    """4th trace field: 'R:'(redex head) 'D:'(discarded root) 'C:src:dst'(copy)
    'A:'(redex argument root, for the directional cue)."""
    redex=None; drop=[]; copy=[]; args=[]; rowlbl=None; hold=0.0; tween=0.0
    for tok in s.split():
        if   tok.startswith("ROWLBL:"): rowlbl=tok[len("ROWLBL:"):].split("|")  # intro row: per-column labels ('|' sep)
        elif tok.startswith("HOLD:"):   hold=float(tok[len("HOLD:"):])          # min hold for this frame (s)
        elif tok.startswith("TWEEN:"):  tween=float(tok[len("TWEEN:"):])        # min morph OUT of this frame (s)
        elif tok.startswith("R:"): redex=int(tok[2:])
        elif tok.startswith("D:"): drop.append(int(tok[2:]))
        elif tok.startswith("C:"): a,b=tok[2:].split(":"); copy.append((int(a),int(b)))
        elif tok.startswith("A:"): args.append(int(tok[2:]))
    return {"redex":redex,"drop":drop,"copy":copy,"args":args,"rowlbl":rowlbl,"hold":hold,"tween":tween}

def _depth(p): return p[4] if len(p)==5 else p[1]    # node depth, for either layout form

def node_xy(pos,nid,cx,cy):
    p=pos[nid]
    if len(p)==5:                                    # row layout: ('R', col, ncols, angle, depth)
        _,col,k,ang,d=p; colx=(2*cx)*(col+1)/(k+1.0); r=d*RSTEP*ROW_SCALE
        return (colx+r*math.cos(ang), cy+r*math.sin(ang))
    ang,d=p; r=d*RSTEP; return (cx+r*math.cos(ang), cy+r*math.sin(ang))

def _subtree(root,kids):
    out=[]; st=[root]
    while st:
        n=st.pop()
        if n not in kids: continue
        out.append(n); st.extend(kids[n])
    return out

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
    copyset=set(); csrc_roots=set()
    for s,d in prov["copy"]:
        copyset.update(_subtree(d,kB)); csrc_roots.add(s)  # the copy subtree -> flashes; original untouched
    newsrc={}
    for nid in inB-inA:                                   # new nodes AND copies emerge from attachment
        p=parB.get(nid)
        while p is not None and p not in inA: p=parB.get(p)
        newsrc[nid]=node_xy(pA,p,cx,cy) if p is not None else (cx,cy)
    # a rearranged argument (kept, not the copied original) that moves far -> trail it
    trail=[]
    for r in prov.get("args",[]):
        if r in inB and r not in csrc_roots:
            ax,ay=node_xy(pA,r,cx,cy); bx,by=node_xy(pB,r,cx,cy)
            if (ax-bx)**2+(ay-by)**2 > TRAIL_THRESH*TRAIL_THRESH: trail.append(r)
    return {"newsrc":newsrc,"gray":gray,"copyset":copyset,"trail":trail,
            "redex":prov.get("redex")}

def frame_prims(W,H, A,B, t, maxd, tr):
    """Backend-agnostic draw list for one morph frame: ('L',x0,y0,x1,y1,hexcol,alpha,width)
    and ('C',x,y,r,hexcol,alpha).  Shared by the SVG and the numpy rasteriser so both
    draw exactly the same thing.  Text is added by the SVG backend / burnt in by ffmpeg."""
    pA,labA,kA,parA,nlA,mdA = A
    pB,labB,kB,parB,nlB,mdB = B
    cx=W/2.0; cy=CAPTOP + (H-CAPTOP)/2.0
    def px(pos,nid): return node_xy(pos,nid,cx,cy)
    te=ease(t)
    newsrc=tr.get("newsrc",{}); gray=tr.get("gray",set())
    copyset=tr.get("copyset",set()); trail=tr.get("trail",[]); redex=tr.get("redex")
    cff=1.0 if t<=0.45 else max(0.0,(0.7-t)/0.25)   # copy-flash: shine bright, then settle by 0.7
    inA=set(pA); inB=set(pB); P=[]
    dcol=[hsl_hex(360.0*d/max(1,maxd),0.85,0.62) for d in range(maxd+1)]  # rainbow by depth, memoised
    _cc={}
    def cur(nid):                          # (x, y, alpha, depth), cached per frame
        v=_cc.get(nid)
        if v is not None: return v
        v=_cur(nid); _cc[nid]=v; return v
    def _cur(nid):
        if nid in inB:
            tgt=px(pB,nid)
            if nid in copyset:                                # a copy: emerge fast, then shine, then settle
                src=newsrc.get(nid,tgt); tc=ease(min(1.0, t/0.28))
                return (lerp(src[0],tgt[0],tc), lerp(src[1],tgt[1],tc), min(1.0, t/0.12), pB[nid][1])
            if nid in inA: src=px(pA,nid); a=1.0              # persists -> glide
            else:          src=newsrc.get(nid,tgt); a=te      # new machinery -> grow from attachment
            return (lerp(src[0],tgt[0],te), lerp(src[1],tgt[1],te), a, _depth(pB[nid]))
        s=px(pA,nid)                                          # dropped -> drift outward as it fades
        dx,dy=s[0]-cx, s[1]-cy; L=math.hypot(dx,dy) or 1.0; k=DRIFT*te
        return (s[0]+dx/L*k, s[1]+dy/L*k, 1-te, _depth(pA[nid]))
    def ncol(nid,d,inB_):                  # edge/dot colour by semantic role
        if (not inB_) and nid in gray: return GRAYC
        if nid in copyset:             return mix(dcol[d], CPFLASH, cff)  # spine flash, fading to normal
        return dcol[d]
    def edges(ids,kids,inB_):
        L=labB if inB_ else labA
        for nid in ids:
            if L.get(nid)=="ROW": continue            # invisible row root: don't connect the columns
            x0,y0,a0,_=cur(nid)
            for c in kids[nid]:
                x1,y1,a1,d1=cur(c); a=min(a0,a1)*0.85
                if a>0.02:
                    w=1.0+2.2*cff if c in copyset else 1.0
                    P.append(('L',x0,y0,x1,y1,ncol(c,d1,inB_),a,w))
    edges(inB,kB,True); edges(inA-inB,kA,False)
    for r in trail:                        # directional cue: faint path of a rearranged arg
        ax,ay=px(pA,r); bx,by,_,_=cur(r); top=0.22*math.sin(math.pi*t)
        if top>0.01:
            P.append(('L',ax,ay,bx,by,TRAILC,top,1.0))
            P.append(('C',bx,by,2.2,TRAILC,min(1.0,top*1.4)))
    for nid in inB | (inA-inB):            # iota leaf dots only
        inB_=nid in inB
        label = labB[nid] if inB_ else labA[nid]
        if label!="1": continue
        x,y,a,d=cur(nid)
        if a>0.02: P.append(('C',x,y,2.4,ncol(nid,d,inB_),a))
    if redex is not None and redex in pA:  # twinkling sparkle where the rule fired (lingers ~3/5 of the morph)
        rx,ry=px(pA,redex); fa=max(0.0, 1.0-1.7*t)
        if fa>0.02:
            c0=0.55+0.45*math.sin(t*40)    # central dot twinkle
            P.append(('C',rx,ry,5.0*(0.7+0.3*c0),FLASH,min(1.0,fa*0.8*c0)))
            for k,(ox,oy,sz) in enumerate(((18,-10,9.0),(-15,13,7.5),(12,17,6.5),(-17,-13,6.0))):
                tw=0.5+0.5*math.sin(t*34 + k*1.7)         # each star out of phase -> scintillation
                op=fa*0.7*tw; s=sz*(0.45+0.55*tw); sx,sy=rx+ox,ry+oy
                if op>0.02:
                    P.append(('L',sx-s,sy,sx+s,sy,FLASH,op,1.6)); P.append(('L',sx,sy-s,sx,sy+s,FLASH,op,1.6))
    return P

def frame_svg(W,H, A,B, t, caption, term, expl, maxd, tr=None):
    out=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" font-family="monospace">',
         f'<rect width="{W}" height="{H}" fill="{BG}"/>',
         f'<text x="{W/2}" y="50" font-size="34" fill="{FG}" text-anchor="middle">{esc(caption)}</text>']
    if term: out.append(f'<text x="{W/2}" y="90" font-size="26" fill="{MUT}" text-anchor="middle">{esc(term)}</text>')
    if expl: out.append(f'<text x="{W/2}" y="128" font-size="32" fill="{ACC}" text-anchor="middle">{esc(expl)}</text>')
    for p in frame_prims(W,H,A,B,t,maxd,tr or {}):
        if p[0]=='L':
            _,x0,y0,x1,y1,col,a,w=p
            out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="{col}" stroke-width="{w:.1f}" opacity="{a:.2f}"/>')
        else:
            _,x,y,r,col,a=p
            out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="{col}" opacity="{a:.2f}"/>')
    out.append("</svg>")
    return "\n".join(out)

# ---------------------------------------------------------------- numpy rasteriser
# Draw the same primitives straight into an RGB buffer (lighten/max blend on the dark
# background) so we can stream frames to ffmpeg and skip the per-file SVG->PNG step.
# All sample points are gathered, then composited in ONE scatter via sort+reduceat
# (numpy's unbuffered ufunc.at is ~10x slower).
import numpy as np
def _scatter_max(img, ys, xs, cols):
    if len(ys)==0: return
    H,W,_=img.shape; flat=img.reshape(-1,3)
    idx=(ys*W+xs).astype(np.int64)
    order=np.argsort(idx, kind='stable'); idx=idx[order]; cols=cols[order]
    uniq, start=np.unique(idx, return_index=True)
    mx=np.maximum.reduceat(cols, start, axis=0)
    flat[uniq]=np.maximum(flat[uniq], mx)

def raster(prims, W, H, gs=1.0):
    # W,H are the OUTPUT pixel dims; gs scales the (native) primitive coords into them,
    # so we render straight at display size instead of big-then-downscale.
    img=np.empty((H,W,3),dtype=np.float32); img[:]=_hex(BG)
    Ls=[p for p in prims if p[0]=='L']; Cs=[p for p in prims if p[0]=='C']
    YS=[]; XS=[]; CC=[]
    if Ls:
        a=np.array([(p[1],p[2],p[3],p[4],p[6],p[7]) for p in Ls],dtype=np.float64)  # x0,y0,x1,y1,alpha,width
        a[:,:4]*=gs
        col=np.array([_hex(p[5]) for p in Ls],dtype=np.float32)*a[:,4,None]         # premultiplied
        dx=a[:,2]-a[:,0]; dy=a[:,3]-a[:,1]; ln=np.hypot(dx,dy)
        S=np.maximum(2,np.ceil(ln).astype(np.int64)+1)
        tot=int(S.sum()); li=np.repeat(np.arange(len(Ls)),S)
        within=np.arange(tot)-(np.cumsum(S)-S)[li]; tt=within/np.maximum(1,(S-1))[li]
        inv=1.0/np.maximum(ln,1e-6); nx=-dy*inv; ny=dx*inv          # perpendicular, to fatten wide lines
        bx=a[li,0]+dx[li]*tt; by=a[li,1]+dy[li]*tt; cc=col[li]; ww=a[li,5]
        for off in (0.0, 0.6, -0.6, 1.2, -1.2):
            m=ww>=(abs(off)*1.6)                                    # only fatten lines wide enough for this rail
            xs=np.round(bx[m]+nx[li[m]]*off).astype(np.int64); ys=np.round(by[m]+ny[li[m]]*off).astype(np.int64)
            ok=(xs>=0)&(xs<W)&(ys>=0)&(ys<H)
            XS.append(xs[ok]); YS.append(ys[ok]); CC.append(cc[m][ok])
    def disk(cx,cy,rad,premult):
        rr=int(math.ceil(rad)); gy,gx=np.mgrid[-rr:rr+1,-rr:rr+1]
        msk=gx*gx+gy*gy<=rad*rad; ox=gx[msk]; oy=gy[msk]
        px=np.round(cx).astype(np.int64)[:,None]+ox[None,:]; py=np.round(cy).astype(np.int64)[:,None]+oy[None,:]
        ok=(px>=0)&(px<W)&(py>=0)&(py<H)
        pm=np.repeat(premult,ox.size,axis=0).reshape(len(cx),ox.size,3)
        XS.append(px[ok]); YS.append(py[ok]); CC.append(pm[ok])
    dots=[p for p in Cs if abs(p[3]-2.4)<1e-6]
    if dots:
        cxv=np.array([p[1] for p in dots])*gs; cyv=np.array([p[2] for p in dots])*gs
        pm=np.array([_hex(p[4]) for p in dots],dtype=np.float32)*np.array([p[5] for p in dots],dtype=np.float32)[:,None]
        disk(cxv,cyv,max(1.4,2.4*gs),pm)
    for p in Cs:
        if abs(p[3]-2.4)<1e-6: continue
        disk(np.array([p[1]*gs]),np.array([p[2]*gs]),max(1.0,p[3]*gs),np.array([_hex(p[4])],dtype=np.float32)*p[5])
    if YS: _scatter_max(img, np.concatenate(YS), np.concatenate(XS), np.concatenate(CC))
    np.clip(img,0,255,out=img)
    return img.astype(np.uint8)

# Frames are independent, so render them across a process pool.  The big shared data
# (layouts, transition roles) is put in a module global before the pool forks, so
# workers inherit it copy-on-write -- only the frame index is passed per task.
_G={}
def _render_frame(f):
    g=_G; Ai,Bi,t,tri=g["specs"][f]
    tr=g["trs"][tri] if tri>=0 else {}
    return raster(frame_prims(g["W"],g["H"],g["lays"][Ai],g["lays"][Bi],t,g["maxd"],tr),
                  g["OW"],g["OH"],g["gs"]).tobytes()

# ---------------------------------------------------------------- main
# Three output modes, all sharing the same schedule + geometry:
#   morph_render.py OUTDIR [secs] [fps]            -> numbered SVG frames (the old way)
#   morph_render.py --dims CAPFILE [secs] [fps]    -> print "W H NFRAMES"; write a
#                                                     caption-timing manifest to CAPFILE
#   morph_render.py --raw [secs] [fps]             -> stream raw rgb24 frames to stdout
# The --dims + --raw pair lets a driver pipe frames straight into ffmpeg (no per-file
# SVG->PNG) and burn the captions with ffmpeg drawtext from the manifest.
def main():
    av=sys.argv[1:]
    mode="svg"; outdir=None; capfile=None
    if av and av[0]=="--raw":   mode="raw"; av=av[1:]
    elif av and av[0]=="--dims": mode="dims"; capfile=av[1]; av=av[2:]
    else:                        outdir=av[0]; av=av[1:]
    target=float(av[0]) if av else 80.0
    fps=int(av[1]) if len(av)>1 else 30
    if outdir: os.makedirs(outdir,exist_ok=True)

    # One pass over the trace.  The terms can be ~1e6 chars each (millions of nodes), so
    # we DON'T parse them up front: node count + max depth come from a paren scan, and the
    # captions only need the rule/term text.  The (huge) sx strings are kept only when a
    # later stage actually builds trees from them, and only for the steps it needs.
    keep_sx = (mode!="dims")
    rules=[]; terms=[]; sxs=[]; pvs=[]; sizes=[]; maxd=0
    for line in sys.stdin:
        f=line.rstrip("\n").split("\t")
        rule = f[0] if f else ""
        if   len(f)>=3: term,sx,pv = f[1],f[2],(f[3] if len(f)>3 else "")
        elif len(f)==2: term,sx,pv = "",f[1],""
        else:           rule,term,sx,pv = "","",(f[0] if f else ""),""
        nodes,d=scan_sx(sx)
        if d>maxd: maxd=d
        rules.append(rule); terms.append(term[:120]); pvs.append(pv); sizes.append(nodes)
        if keep_sx: sxs.append(sx)
    n=len(rules)
    maxd=max(maxd, int(os.environ.get("MORPH_MAXD","0")))   # force canvas depth so separate clips match scale
    SZ=2*(maxd+1)*RSTEP
    W=int(SZ+120); H=int(CAPTOP+SZ+50); W+=W&1; H+=H&1   # native canvas
    # render straight at display size (the streaming path); SVG keeps native.
    MAXW=1000
    gs=min(1.0, MAXW/W) if mode in ("raw","dims") else 1.0
    OW=int(W*gs); OH=int(H*gs); OW+=OW&1; OH+=OH&1

    steps=list(zip(rules,terms))   # schedule/captions only read [0]=rule and [1]=term
    def term_for(i):
        s=terms[i].strip()
        return s if len(s)<=72 else s[:69]+"..."
    provs=[parse_prov(pvs[i]) for i in range(n)]
    hold_hint=[p.get("hold",0.0) for p in provs]; tween_hint=[p.get("tween",0.0) for p in provs]
    hold, tween, starts = schedule(steps, sizes, target, fps, hold_hint, tween_hint)

    if mode=="dims":
        # per-frame caption text (no geometry needed), coalesced into [start,end) windows
        def caps_iter():
            for i in range(n):
                cap=caption_call(rules[i])[0]; term=term_for(i); expl=step_expl(steps[i])
                for _ in range(hold[i]): yield (cap,term,expl)
                if i+1<n:
                    cap2=caption_call(rules[i+1])[0]; term2=term_for(i+1); expl2=step_expl(steps[i+1])
                    tw=tween[i]
                    for fc in range(tw):
                        late=(fc+1)/tw>0.5
                        yield (cap2 if late else cap, term2 if late else term, expl2 if late else expl)
        rows=list(caps_iter()); total=len(rows)
        segs=[]   # (lineidx, startframe, endframe, text); line 0=caption,1=term,2=expl
        for li in range(3):
            cur_txt=None; cs=0
            for fi,row in enumerate(rows):
                txt=row[li]
                if txt!=cur_txt:
                    if cur_txt: segs.append((li,cs,fi,cur_txt))
                    cur_txt=txt; cs=fi
            if cur_txt: segs.append((li,cs,total,cur_txt))
        # intro "row" frames: a value label under each column (6-field 'L' lines; the x
        # is a width fraction so it survives the gs scale).  Linger a little into the morph.
        labels=[]   # ("L", start_s, end_s, text, xfrac, y_out)
        for i in range(n):
            lbls=parse_prov(pvs[i]).get("rowlbl")
            if not lbls: continue
            k=len(lbls); a=starts[i]/fps
            b=(starts[i]+hold[i]+(tween[i]//3 if i+1<n else 0))/fps
            ly=round(0.66*OH)
            for col,txt in enumerate(lbls):
                if not txt: continue                       # list columns carry no label
                labels.append(("L", a, b, txt, (col+1)/(k+1.0), ly))
        with open(capfile,"w") as cf:
            for li,s,e,txt in segs:
                cf.write(f"{li}\t{s/fps:.3f}\t{e/fps:.3f}\t{txt}\n")
            for _,a,b,txt,xf,ly in labels:
                cf.write(f"L\t{a:.3f}\t{b:.3f}\t{txt}\t{xf:.4f}\t{ly}\n")
        sys.stderr.write(f"maxd={maxd}\n")             # so a driver can match this canvas on another clip
        print(f"{OW} {OH} {total} {gs:.5f}")
        return

    # flat frame table -- (Aidx, Bidx, t, transition-index) -- built from int counts only.
    specs=[]; tr_pairs=[]                              # tr_pairs[tri] = the step i of transition i->i+1
    for i in range(n):
        for _ in range(hold[i]): specs.append((i,i,0.0,-1))
        if i+1<n:
            tri=len(tr_pairs); tr_pairs.append(i)
            for fc in range(tween[i]): specs.append((i,i+1,(fc+1)/tween[i],tri))
    total=len(specs)

    if mode=="raw":
        start=int(av[2]) if len(av)>2 else 0           # optional frame window [start,start+count)
        count=int(av[3]) if len(av)>3 else total       # lets a driver render segments independently
        lo=max(0,start); hi=min(start+count, total)
        # only the steps/transitions this window shows need their trees built + laid out
        need_steps=set(); need_tris=set()
        for (Ai,Bi,_t,tri) in specs[lo:hi]:
            need_steps.add(Ai); need_steps.add(Bi)
            if tri>=0: need_tris.add(tri)
        lays={i:layout(parse(sxs[i])) for i in need_steps}
        trs={tri:compute_transition(W,H,lays[tr_pairs[tri]],lays[tr_pairs[tri]+1],parse_prov(pvs[tr_pairs[tri]+1]))
             for tri in need_tris}
        sxs.clear()                                    # free the ~hundreds-of-MB of term strings before forking
        _G.update(lays=lays,trs=trs,specs=specs,W=W,H=H,OW=OW,OH=OH,gs=gs,maxd=maxd)
        idxs=range(lo,hi)
        jobs=max(1,int(os.environ.get("MORPH_JOBS","8")))
        buf=sys.stdout.buffer
        try:
            if jobs==1:
                for f in idxs: buf.write(_render_frame(f))
            else:
                import multiprocessing as mp
                with mp.Pool(jobs) as pool:                  # workers fork *after* _G is set
                    for b in pool.imap(_render_frame, idxs, chunksize=4):
                        buf.write(b)
        except BrokenPipeError:
            pass                                             # consumer (ffmpeg/head) closed early
        return

    # svg: numbered files, one per frame (the old path) -- builds every layout up front
    lays=[layout(parse(sxs[i])) for i in range(n)]
    fnum=0
    for i in range(n):
        cap=caption_call(rules[i])[0]; term=term_for(i); expl=step_expl(steps[i])
        for _ in range(hold[i]):
            open(os.path.join(outdir,f"f{fnum:05d}.svg"),"w").write(frame_svg(W,H,lays[i],lays[i],0.0,cap,term,expl,maxd,{})); fnum+=1
        if i+1<n:
            cap2=caption_call(rules[i+1])[0]; term2=term_for(i+1); expl2=step_expl(steps[i+1])
            tr=compute_transition(W,H,lays[i],lays[i+1], parse_prov(pvs[i+1])); tw=tween[i]
            for fc in range(tw):
                t=(fc+1)/tw; late=t>0.5
                open(os.path.join(outdir,f"f{fnum:05d}.svg"),"w").write(
                    frame_svg(W,H,lays[i],lays[i+1],t, cap2 if late else cap,
                              term2 if late else term, expl2 if late else expl, maxd, tr)); fnum+=1
    print(fnum)

if __name__=="__main__":
    main()
