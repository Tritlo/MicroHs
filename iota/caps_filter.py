#!/usr/bin/env python3
"""Turn morph_render's --dims caption manifest into an ffmpeg drawtext filter chain.

  caps_filter.py CAPS.tsv TEXTDIR FONT [total_seconds] [geom_scale]

Reads lines "lineidx<TAB>start_s<TAB>end_s<TAB>text", writes each text to TEXTDIR
(so no shell escaping of the content is needed), and prints a comma-joined chain of
drawtext filters (one per segment), each shown only during its [start,end) window.
Segments that run to the end are extended a little so the final caption survives the
freeze frame.  Line 0 = rule caption, 1 = machine-state term, 2 = green insight."""
import sys, os

STYLE={0:("#e6edf3",34,26), 1:("#6e7681",26,70), 2:("#7ee787",32,106)}  # colour, size, y

def main():
    caps, td, font = sys.argv[1], sys.argv[2], sys.argv[3]
    total=float(sys.argv[4]) if len(sys.argv)>4 else None
    gs=float(sys.argv[5]) if len(sys.argv)>5 else 1.0           # video rendered at gs*native
    os.makedirs(td, exist_ok=True)
    parts=[]
    for i,ln in enumerate(open(caps)):
        f=ln.rstrip("\n").split("\t")
        if len(f)<4: continue
        li=int(f[0]); a=float(f[1]); b=float(f[2]); txt=f[3]
        if total is not None and b>=total-1e-3: b=total+4.0      # persist through the end/freeze
        col,sz,y=STYLE[li]; sz=max(10,round(sz*gs)); y=round(y*gs)
        tf=os.path.join(td,f"s{i}.txt"); open(tf,"w").write(txt)
        parts.append(f"drawtext=fontfile={font}:textfile={tf}:fontcolor={col}:fontsize={sz}:"
                     f"x=(w-text_w)/2:y={y}:enable=between(t\\,{a:.3f}\\,{b:.3f})")
    sys.stdout.write(",".join(parts))

if __name__=="__main__":
    main()
