#!/usr/bin/env python3
"""Render a reduction as a smooth, semantic morphing film with a generated soundtrack.

  morph_film.py MODULE ROOT OUT.mp4 [seconds] [fps]
    defaults: Lt  Lt.test  iota-examples/films/morph_lt23.mp4  120  30

This is the render pipeline driver; the heavy lifting stays in the helper scripts
(functional core, imperative shell):
  * bin/gmhs -ddump-combinator dumps MODULE; iota/morph turns ROOT's combinator def into
    an id-tracked reduction trace (with provenance);
  * morph_render.py --dims sizes the canvas + writes the caption-timing manifest;
  * the timeline is split into SEG_SECS chunks; up to SEG_JOBS render at once, each
    streaming rgb24 frames from morph_render.py --raw (MORPH_JOBS frame workers) into its
    own ffmpeg, burning just that chunk's captions (caps_filter.py) -- which also keeps
    each segment's caption filtergraph small, so long reductions don't blow ffmpeg's
    command line;
  * the chunks are concatenated (stream copy) and the soundtrack (morph_music.py) muxed in.
Knobs (env): SEG_SECS (30), SEG_JOBS (4), MORPH_JOBS (4), MUSIC_MODE (comb), MORPH_FONT.
Needs: bin/gmhs, ghc, python3 (numpy), ffmpeg.  (No ImageMagick.)
"""
import os, sys, shutil, subprocess, tempfile
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # repo root (parent of iota/)
os.chdir(ROOT)
PY = sys.executable
PROG = "iota-examples/programs"

def _tail(path, n):
    try:
        with open(path) as f: return "".join(f.readlines()[-n:])
    except OSError: return ""

def main():
    a = sys.argv[1:]
    mod  = a[0] if len(a)>0 else "Lt"
    defn = a[1] if len(a)>1 else "Lt.test"
    out  = a[2] if len(a)>2 else "iota-examples/films/morph_lt23.mp4"
    secs = a[3] if len(a)>3 else "120"
    fps  = int(a[4]) if len(a)>4 else 30
    seg_secs   = int(os.environ.get("SEG_SECS", "30"))
    seg_jobs   = int(os.environ.get("SEG_JOBS", "4"))
    morph_jobs = os.environ.get("MORPH_JOBS", "4")
    music_mode = os.environ.get("MUSIC_MODE", "comb")
    font = os.environ.get("MORPH_FONT", "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf")

    W = tempfile.mkdtemp()
    os.makedirs(os.path.dirname(out) or ".", exist_ok=True)
    try:
        # build the id-tracked reducer if needed
        if not os.access("iota/morph", os.X_OK):
            subprocess.run(["ghc","-O0","-outputdir",os.path.join(W,"b"),"-o","iota/morph","iota/Morph.hs"], check=True)
        # gmhs dumps the combinator program, then exits non-zero (no `main`) -- expected
        raw = subprocess.run([f"{ROOT}/bin/gmhs", f"-i{PROG}", "-ilib", "-ddump-combinator", mod],
                             capture_output=True, text=True).stdout
        dump = os.path.join(W,"dump")
        with open(dump,"w") as f:
            f.writelines(l+"\n" for l in raw.splitlines() if l.startswith(mod+"."))
        # reduce ROOT into an id-tracked trace
        trace = os.path.join(W,"trace.txt")
        with open(trace,"wb") as f:
            subprocess.run(["iota/morph","--iota",dump,defn], stdout=f, check=True)

        # output canvas size, frame count, geometry scale + caption-timing manifest
        caps = os.path.join(W,"caps.tsv")
        with open(trace,"rb") as f:
            dims = subprocess.run([PY,"iota/morph_render.py","--dims",caps,secs,str(fps)],
                                  stdin=f, capture_output=True, text=True, check=True).stdout.split()
        vw, vh, nf, gs = dims[0], dims[1], int(dims[2]), dims[3]
        total = nf/fps
        segf = seg_secs*fps
        nseg = (nf + segf - 1)//segf
        print(f"frames: {nf} (~{total:.3f}s @ {fps}fps), {vw}x{vh}; "
              f"{nseg} segment(s) x {seg_secs}s, {seg_jobs} parallel x {morph_jobs} workers", flush=True)

        # soundtrack on the same schedule
        music = os.path.join(W,"music.wav")
        with open(trace,"rb") as f:
            subprocess.run([PY,"iota/morph_music.py",music,secs,str(fps),music_mode],
                           stdin=f, stdout=subprocess.DEVNULL, check=True)

        def render_seg(k):                       # render+encode chunk k (frames [start, start+count))
            start = k*segf; count = min(nf-start, segf)
            tstart = start/fps; last = (k == nseg-1)
            tend = (total+5) if last else (start+count)/fps
            pre  = "tpad=stop_mode=clone:stop_duration=2.5" if last else ""   # 2.5s hold on the result
            filt = subprocess.run([PY,"iota/caps_filter.py",caps,os.path.join(W,f"td{k}"),
                                   font,f"{total:.3f}",gs,f"{tstart:.3f}",f"{tend:.3f}"],
                                  capture_output=True, text=True, check=True).stdout
            body = ",".join(p for p in (pre, filt) if p) or "null"
            fc = os.path.join(W,f"fc{k}")
            with open(fc,"w") as f: f.write(f"[0:v]{body}[v]")
            seg = os.path.join(W,f"seg{k}.mp4")
            with open(os.path.join(W,f"ff{k}.log"),"wb") as log, open(trace,"rb") as tf:
                rend = subprocess.Popen([PY,"iota/morph_render.py","--raw",secs,str(fps),str(start),str(count)],
                                        stdin=tf, stdout=subprocess.PIPE, env=dict(os.environ, MORPH_JOBS=str(morph_jobs)))
                ff = subprocess.Popen(["ffmpeg","-y","-f","rawvideo","-pix_fmt","rgb24","-s",f"{vw}x{vh}",
                                       "-framerate",str(fps),"-i","-","-filter_complex_script",fc,"-map","[v]",
                                       "-c:v","libx264","-crf","20","-preset","medium","-pix_fmt","yuv420p","-an",seg],
                                      stdin=rend.stdout, stderr=log)
                rend.stdout.close()              # let the renderer see EOF/SIGPIPE if ffmpeg exits early
                ff.wait(); rend.wait()

        with ThreadPoolExecutor(max_workers=seg_jobs) as ex:
            list(ex.map(render_seg, range(nseg)))

        # concat the chunks (stream copy), then mux the soundtrack
        listf = os.path.join(W,"list.txt")
        with open(listf,"w") as lf:
            for k in range(nseg):
                seg = os.path.join(W,f"seg{k}.mp4")
                if not (os.path.exists(seg) and os.path.getsize(seg) > 0):
                    sys.stderr.write(f"segment {k} failed:\n" + _tail(os.path.join(W,f"ff{k}.log"),3))
                    sys.exit(1)
                lf.write(f"file '{seg}'\n")
        silent = os.path.join(W,"silent.mp4")
        subprocess.run(["ffmpeg","-y","-f","concat","-safe","0","-i",listf,"-c","copy",silent],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
        subprocess.run(["ffmpeg","-y","-i",silent,"-i",music,"-map","0:v","-map","1:a",
                        "-c:v","copy","-c:a","aac","-b:a","192k","-shortest",out],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
        print(f"wrote {out}")
    finally:
        shutil.rmtree(W, ignore_errors=True)

if __name__=="__main__":
    main()
