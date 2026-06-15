#!/usr/bin/env python3
"""Sonify an identity-tracked reduction trace into a WAV, in lock-step with the
morph film (it shares iota/morph_render's `schedule`, so notes land exactly on the
morphs).  Each step is a note on a pentatonic scale whose pitch follows the tree
size -- the mandala expanding makes the melody rise, the collapse to the answer
makes it fall to the tonic.  The Y (recursion) steps get a low bass accent and the
final A = True resolves on a sustained tonic chord, over a soft root drone.

  morph_music.py OUT.wav [target_seconds] [fps]   (trace on stdin)
"""
import sys, os, math, wave, struct
from array import array
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import morph_render as mr

SR=44100
ROOT=261.63                       # C4
PENTA=[0,2,4,7,9]                 # major pentatonic, semitones within an octave
MELODY=-12                        # transpose the melody + final chord down an octave
                                  # (warmer register); the bass/drone stay as anchors

# "comb" pitch mode: each combinator *family* shares a pitch class, and a prime
# lifts it an octave -- S=G4/S'=G5, C=D4/C'=D5, B=A4 -- which mirrors the maths (a
# prime is the same combinator threading one more argument).  K/I are the tonic, J
# the third.  We cap the climb at one octave so nothing gets shrill: the doubly-
# decorated C'B shares C''s register, and the lone R shares S'.  Semitones above C4.
COMB_SEMI={"":0,"I":0,"K":0,"K2":12,"K3":12,"K4":12,"U":12,
           "C":2,"C'":14,"C'B":14,"J":4,"Z":16,
           "S":7,"S'":19,"R":19,"B":9,"B'":21,"Y":0}

def freq(semi): return ROOT*2.0**(semi/12.0)
def deg_semi(d): return 12*(d//5)+PENTA[d%5]      # pentatonic degree -> semitone

def add(buf, t0, dur, f0, amp, harm=(1.0,0.28,0.1), decay=None):
    """Mix a bell-ish tone (sine + a couple of harmonics, 5 ms attack, exponential
    decay) into the float buffer."""
    n=len(buf); i0=int(t0*SR); N=int(dur*SR); decay=decay or dur
    for k in range(N):
        idx=i0+k
        if idx<0 or idx>=n: continue
        t=k/SR
        env=math.exp(-3.0*t/decay)*(1.0-math.exp(-t/0.005))
        s=0.0
        for hi,ha in enumerate(harm, start=1):
            s+=ha*math.sin(2.0*math.pi*f0*hi*t)
        buf[idx]+=amp*env*s

def main():
    out=sys.argv[1]
    target=float(sys.argv[2]) if len(sys.argv)>2 else 88.0
    fps=int(sys.argv[3]) if len(sys.argv)>3 else 30
    mode=sys.argv[4] if len(sys.argv)>4 else "size"   # "size": pitch follows the
    #   tree size; "comb": each combinator drives the pitch (a motif per rule)
    steps=[]
    for line in sys.stdin:
        f=line.rstrip("\n").split("\t")
        if len(f)>=3:   rule,term,sx=f[0],f[1],f[2]
        elif len(f)==2: rule,term,sx=f[0],"",f[1]
        else:           rule,term,sx="","",f[0]
        steps.append((rule,term,sx))
    n=len(steps)
    sizes=[s[2].count("(") for s in steps]            # node count == '(' count, as in the renderer
    hold,tween,starts=mr.schedule([(s[0],s[1]) for s in steps], sizes, target, fps)
    total_dur=(starts[-1]+hold[-1])/fps

    # pitch: either follows the tree size (log-quantised to ~3 octaves of the
    # pentatonic -- small/answer = tonic, peak mandala = top), or is fixed per
    # combinator so each rule has its own note.
    lo=math.log2(min(sizes)); hi=math.log2(max(sizes)); span=max(1e-6,hi-lo); DMAX=14
    def size_deg(sz): return round((math.log2(sz)-lo)/span*DMAX)
    def comb_head(i):
        p=steps[i][0].split(); return p[0] if p else ""
    def note_semi(i):
        if mode=="comb": return COMB_SEMI.get(comb_head(i),0)
        return deg_semi(size_deg(sizes[i]))

    buf=array('d',[0.0])*int((total_dur+3.0)*SR)      # +3 s for the final chord to ring
    # soft root drone for tonal glue
    add(buf, 0.0, total_dur+2.0, freq(-24), 0.05, harm=(1.0,0.0), decay=(total_dur+2.0)*1.4)
    for i in range(n):
        t0=starts[i]/fps
        ring=min((hold[i]+(tween[i] if i+1<n else 0))/fps + 0.9, 2.2)
        if i==n-1:                                    # A = True -> sustained tonic chord
            for semi in (0,4,7,12):
                add(buf, t0, 2.8, freq(semi+MELODY), 0.22, decay=2.6)
            continue
        add(buf, t0, ring, freq(note_semi(i)+MELODY), 0.26)
        if steps[i][0].startswith("Y "):              # recursion: low bass accent
            add(buf, t0, min(ring+0.6,2.6), freq(-24+PENTA[0]), 0.40, harm=(1.0,0.5,0.0), decay=1.8)

    peak=max(1e-6, max(abs(x) for x in buf))
    g=0.89/peak
    w=wave.open(out,"w"); w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
    w.writeframes(b"".join(struct.pack("<h", int(max(-1.0,min(1.0,x*g))*32767)) for x in buf))
    w.close()
    print(f"{out}  {total_dur:.1f}s  {n} notes  ({mode})")

if __name__=="__main__":
    main()
