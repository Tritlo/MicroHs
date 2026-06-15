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

    # pitch = tree size (log) quantised to ~3 octaves of the pentatonic; small tree
    # (the 11-node answer) -> tonic, the peak mandala -> top of the range.
    lo=math.log2(min(sizes)); hi=math.log2(max(sizes)); span=max(1e-6,hi-lo); DMAX=14
    def degree(sz): return round((math.log2(sz)-lo)/span*DMAX)

    buf=array('d',[0.0])*int((total_dur+3.0)*SR)      # +3 s for the final chord to ring
    # soft root drone for tonal glue
    add(buf, 0.0, total_dur+2.0, freq(-24), 0.05, harm=(1.0,0.0), decay=(total_dur+2.0)*1.4)
    for i in range(n):
        t0=starts[i]/fps
        ring=min((hold[i]+(tween[i] if i+1<n else 0))/fps + 0.9, 2.2)
        if i==n-1:                                    # A = True -> sustained tonic chord
            for semi in (0,4,7,12):
                add(buf, t0, 2.8, freq(semi), 0.22, decay=2.6)
            continue
        note=freq(deg_semi(degree(sizes[i])))
        add(buf, t0, ring, note, 0.26)
        if steps[i][0].startswith("Y "):              # recursion: low bass accent
            add(buf, t0, min(ring+0.6,2.6), freq(-24+PENTA[0]), 0.40, harm=(1.0,0.5,0.0), decay=1.8)

    peak=max(1e-6, max(abs(x) for x in buf))
    g=0.89/peak
    w=wave.open(out,"w"); w.setnchannels(1); w.setsampwidth(2); w.setframerate(SR)
    w.writeframes(b"".join(struct.pack("<h", int(max(-1.0,min(1.0,x*g))*32767)) for x in buf))
    w.close()
    print(f"{out}  {total_dur:.1f}s  {n} notes")

if __name__=="__main__":
    main()
