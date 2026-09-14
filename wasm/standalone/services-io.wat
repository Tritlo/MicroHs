;; Buffered streams and fixed IO services. No function evaluates a graph or
;; invokes collection. Service arguments are already in weak head normal form.
;;
;; A stream handle is a 64-byte payload block. Its fields are:
;;   0 kind: 1=fd, 2=read memory, 3=write memory, 4=UTF-8, 5=CRLF, 6=directory
;;   4 flags: read=1, write=2, own fd=4, line output=8, immediate output=16
;;   8 fd; 12 buffer; 16 capacity; 20 position; 24 read limit; 28 pushed value
;;   32 stream errno; 36 child stream; 40 buffer mode (0=unused, 1=read, 2=write)
;;   44..51 directory cookie; 52 current directory name; 56..63 reserved
;; Pointer words 3, 9, and 13 give the mask 8712. Memory streams retain their
;; data through the buffer pointer. Read memory does not own its input buffer.
;; Write-memory buffers belong to the caller, as required by get_mem.
;;
;; Byte and UTF-8 streams are separate. The UTF-8 layer accepts modified UTF-8
;; NUL (c0 80), preserves the C runtime's surrogate/continuation policy, and
;; reports truncated sequences as EOF. It rejects other overlong encodings.

(func $bfile_new (param $kind i32) (result i32)
  (local $h i32)
  (local.set $h (call $blob_alloc (i32.const 64) (i32.const 8712)))
  (memory.fill (local.get $h) (i32.const 0) (i32.const 64))
  (i32.store (local.get $h) (local.get $kind))
  (i32.store offset=28 (local.get $h) (i32.const -1))
  (local.get $h))

;; Foreign pointers and byte strings share a descriptor layout. Mode zero
;; identifies a foreign pointer; mode one identifies a byte string.
(func $io_foreign_pointer (param $p i32) (result i32)
  (local $desc i32) (local $n i32)
  (local.set $desc (call $blob_alloc (i32.const 16) (i32.const 2)))
  (memory.fill (local.get $desc) (i32.const 0) (i32.const 16))
  (i32.store offset=4 (local.get $desc) (local.get $p))
  (local.set $n (call $node (global.get $T_FORPTR)))
  (i32.store offset=4 (local.get $n) (local.get $desc))
  (local.get $n))

(func $bfile_fd (param $fd i32) (param $owned i32) (result i32)
  (local $h i32) (local $flags i32)
  (local.set $h (call $bfile_new (i32.const 1)))
  (local.set $flags (i32.or (i32.const 3) (i32.shl (local.get $owned) (i32.const 2))))
  (if (i32.eqz (local.get $owned))
    (then
      (if (i32.eq (local.get $fd) (i32.const 0)) (then (local.set $flags (i32.const 1))))
      (if (i32.eq (local.get $fd) (i32.const 1)) (then (local.set $flags (i32.const 10))))
      (if (i32.eq (local.get $fd) (i32.const 2)) (then (local.set $flags (i32.const 18))))))
  (i32.store offset=4 (local.get $h) (local.get $flags))
  (i32.store offset=8 (local.get $h) (local.get $fd))
  (i32.store offset=12 (local.get $h) (call $blob_alloc (i32.const 16384) (i32.const 0)))
  (i32.store offset=16 (local.get $h) (i32.const 16384))
  (local.get $h))

(func $bfile_rd_mem (param $p i32) (param $n i32) (result i32)
  (local $h i32)
  (local.set $h (call $bfile_new (i32.const 2)))
  (i32.store offset=4 (local.get $h) (i32.const 1))
  (i32.store offset=12 (local.get $h) (local.get $p))
  (i32.store offset=16 (local.get $h) (local.get $n))
  (i32.store offset=24 (local.get $h) (local.get $n))
  (local.get $h))

(func $bfile_wr_mem (result i32)
  (local $h i32)
  (local.set $h (call $bfile_new (i32.const 3)))
  (i32.store offset=4 (local.get $h) (i32.const 2))
  (i32.store offset=12 (local.get $h) (call $blob_alloc (i32.const 4096) (i32.const 0)))
  (i32.store offset=16 (local.get $h) (i32.const 4096))
  (local.get $h))

(func $bfile_utf8 (param $child i32) (result i32)
  (local $h i32)
  (local.set $h (call $bfile_new (i32.const 4)))
  (i32.store offset=36 (local.get $h) (local.get $child))
  (local.get $h))

(func $bfile_crlf (param $child i32) (result i32)
  (local $h i32)
  (local.set $h (call $bfile_new (i32.const 5)))
  (i32.store offset=36 (local.get $h) (local.get $child))
  (local.get $h))

(func $bfile_error (param $h i32) (result i32)
  (if (i32.eqz (local.get $h)) (then (return (i32.const 8))))
  (if (i32.load offset=32 (local.get $h)) (then (return (i32.load offset=32 (local.get $h)))))
  (if (i32.load offset=36 (local.get $h))
    (then (return (call $bfile_error (i32.load offset=36 (local.get $h))))))
  (i32.const 0))

(func $bfile_set_error (param $h i32) (param $error i32) (result i32)
  (if (local.get $h) (then (i32.store offset=32 (local.get $h) (local.get $error))))
  (call $io_errno (local.get $error)))

;; Only fd streams own write buffers. Memory output is already in its buffer.
;; On a short host write, retain the unwritten suffix and report its errno.
(func $bfile_flush (param $h i32)
  (local $kind i32) (local $n i32) (local $wrote i32) (local $buf i32)
  (if (i32.eqz (local.get $h)) (then (return)))
  (local.set $kind (i32.load (local.get $h)))
  (if (i32.load offset=36 (local.get $h))
    (then (call $bfile_flush (i32.load offset=36 (local.get $h))) (return)))
  (if (i32.or (i32.ne (local.get $kind) (i32.const 1))
               (i32.ne (i32.load offset=40 (local.get $h)) (i32.const 2))) (then (return)))
  (local.set $n (i32.load offset=20 (local.get $h)))
  (if (i32.eqz (local.get $n)) (then (return)))
  (local.set $buf (i32.load offset=12 (local.get $h)))
  (local.set $wrote (call $wasi_write_all (i32.load offset=8 (local.get $h)) (local.get $buf) (local.get $n)))
  (if (i32.ne (local.get $wrote) (local.get $n))
    (then
      (drop (call $bfile_set_error (local.get $h) (i32.load (i32.const 0x240))))
      (memory.copy (local.get $buf) (i32.add (local.get $buf) (local.get $wrote))
        (i32.sub (local.get $n) (local.get $wrote)))))
  (i32.store offset=20 (local.get $h) (i32.sub (local.get $n) (local.get $wrote))))

;; Close releases stream metadata and owned fd buffers. It does not free a
;; write-memory buffer; get_mem transfers that buffer to its caller.
(func $bfile_close (param $h i32)
  (local $kind i32) (local $error i32)
  (if (i32.eqz (local.get $h)) (then (return)))
  (local.set $kind (i32.load (local.get $h)))
  (if (i32.eqz (local.get $kind)) (then (return)))
  (call $bfile_flush (local.get $h))
  (if (i32.load offset=36 (local.get $h))
    (then (call $bfile_close (i32.load offset=36 (local.get $h))))
    (else
      (if (i32.or (i32.eq (local.get $kind) (i32.const 1)) (i32.eq (local.get $kind) (i32.const 6)))
        (then
          (if (i32.and (i32.load offset=4 (local.get $h)) (i32.const 4))
            (then
              (local.set $error (call $wasi_fd_close (i32.load offset=8 (local.get $h))))
              (if (local.get $error) (then (drop (call $io_errno (local.get $error)))))))
          (call $blob_free (i32.load offset=12 (local.get $h)))))))
  (if (i32.load offset=52 (local.get $h))
    (then (call $blob_free (i32.load offset=52 (local.get $h)))))
  (i32.store (local.get $h) (i32.const 0))
  (call $blob_free (local.get $h)))

;; Finalizer handles are fixed service indices plus one, never C addresses.
;; The collector calls this before it frees payload blocks. No allocation,
;; evaluation, or collection is permitted on this path.
(func $finalize (param $handle i32) (param $data i32)
  (if (i32.eqz (local.get $handle)) (then (return)))
  (if (i32.eq (local.get $handle) (i32.add (global.get $svc_closeb) (i32.const 1)))
    (then (call $bfile_close (local.get $data)) (return)))
  (if (i32.eq (local.get $handle) (i32.add (global.get $svc_free) (i32.const 1)))
    (then (if (local.get $data) (then (call $blob_free (local.get $data)))) (return)))
  (call $fail (i32.const 88)))

;; Grow an owned memory-output buffer. Callers must not retain get_mem's data
;; pointer across another write; the same rule applies to the C BFILE API.
(func $bfile_reserve (param $h i32) (param $needed i32)
  (local $cap i32) (local $old i32) (local $new i32)
  (local.set $cap (i32.load offset=16 (local.get $h)))
  (if (i32.le_u (local.get $needed) (local.get $cap)) (then (return)))
  (loop $grow
    (if (i32.ge_u (local.get $cap) (i32.const 0x40000000)) (then (call $fail (i32.const 48))))
    (local.set $cap (i32.shl (local.get $cap) (i32.const 1)))
    (br_if $grow (i32.lt_u (local.get $cap) (local.get $needed))))
  (local.set $new (call $blob_alloc (local.get $cap) (i32.const 0)))
  (local.set $old (i32.load offset=12 (local.get $h)))
  (memory.copy (local.get $new) (local.get $old) (i32.load offset=20 (local.get $h)))
  (i32.store offset=12 (local.get $h) (local.get $new))
  (i32.store offset=16 (local.get $h) (local.get $cap))
  (call $blob_free (local.get $old)))

(func $bfile_unget (param $c i32) (param $h i32)
  (local $pos i32)
  (if (i32.lt_s (local.get $c) (i32.const 0)) (then (return)))
  (if (i32.eq (i32.load (local.get $h)) (i32.const 2))
    (then
      (local.set $pos (i32.load offset=20 (local.get $h)))
      (if (i32.eqz (local.get $pos))
        (then (drop (call $bfile_set_error (local.get $h) (i32.const 28))) (return)))
      (local.set $pos (i32.sub (local.get $pos) (i32.const 1)))
      (if (i32.ne (i32.load8_u (i32.add (i32.load offset=12 (local.get $h)) (local.get $pos)))
                  (i32.and (local.get $c) (i32.const 255)))
        (then (drop (call $bfile_set_error (local.get $h) (i32.const 28))) (return)))
      (i32.store offset=20 (local.get $h) (local.get $pos))
      (return)))
  (if (i32.ge_s (i32.load offset=28 (local.get $h)) (i32.const 0))
    (then (drop (call $bfile_set_error (local.get $h) (i32.const 28))) (return)))
  (i32.store offset=28 (local.get $h) (local.get $c)))

;; Decode one code point with the policy in the existing BFILE UTF-8 layer.
;; In particular, continuation bytes are masked as in that implementation.
(func $bfile_get_utf8 (param $h i32) (result i32)
  (local $child i32) (local $a i32) (local $b i32) (local $c i32) (local $d i32) (local $value i32)
  (local.set $child (i32.load offset=36 (local.get $h)))
  (local.set $a (call $bfile_get (local.get $child)))
  (if (i32.lt_s (local.get $a) (i32.const 128)) (then (return (local.get $a))))
  (local.set $b (call $bfile_get (local.get $child)))
  (if (i32.lt_s (local.get $b) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.eq (i32.and (local.get $a) (i32.const 224)) (i32.const 192))
    (then
      (local.set $value (i32.or (i32.shl (i32.and (local.get $a) (i32.const 31)) (i32.const 6))
        (i32.and (local.get $b) (i32.const 63))))
      (if (i32.and (i32.gt_u (local.get $value) (i32.const 0)) (i32.lt_u (local.get $value) (i32.const 128)))
        (then (return (call $bfile_set_error (local.get $h) (i32.const 25)))))
      (return (local.get $value))))
  (local.set $c (call $bfile_get (local.get $child)))
  (if (i32.lt_s (local.get $c) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.eq (i32.and (local.get $a) (i32.const 240)) (i32.const 224))
    (then
      (local.set $value (i32.or (i32.shl (i32.and (local.get $a) (i32.const 15)) (i32.const 12))
        (i32.or (i32.shl (i32.and (local.get $b) (i32.const 63)) (i32.const 6)) (i32.and (local.get $c) (i32.const 63)))))
      (if (i32.lt_u (local.get $value) (i32.const 2048))
        (then (return (call $bfile_set_error (local.get $h) (i32.const 25)))))
      (return (local.get $value))))
  (local.set $d (call $bfile_get (local.get $child)))
  (if (i32.lt_s (local.get $d) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.eq (i32.and (local.get $a) (i32.const 248)) (i32.const 240))
    (then
      (local.set $value (i32.or (i32.shl (i32.and (local.get $a) (i32.const 7)) (i32.const 18))
        (i32.or (i32.shl (i32.and (local.get $b) (i32.const 63)) (i32.const 12))
          (i32.or (i32.shl (i32.and (local.get $c) (i32.const 63)) (i32.const 6)) (i32.and (local.get $d) (i32.const 63))))))
      (if (i32.lt_u (local.get $value) (i32.const 65536))
        (then (return (call $bfile_set_error (local.get $h) (i32.const 25)))))
      (return (local.get $value))))
  (call $bfile_set_error (local.get $h) (i32.const 25)))

(func $bfile_get (param $h i32) (result i32)
  (local $kind i32) (local $c i32) (local $pos i32) (local $got i32) (local $child i32)
  (if (i32.eqz (local.get $h)) (then (return (call $io_errno (i32.const 8)))))
  (local.set $c (i32.load offset=28 (local.get $h)))
  (if (i32.ge_s (local.get $c) (i32.const 0))
    (then (i32.store offset=28 (local.get $h) (i32.const -1)) (return (local.get $c))))
  (local.set $kind (i32.load (local.get $h)))
  (if (i32.eq (local.get $kind) (i32.const 4)) (then (return (call $bfile_get_utf8 (local.get $h)))))
  (if (i32.eq (local.get $kind) (i32.const 5))
    (then
      (local.set $child (i32.load offset=36 (local.get $h)))
      (local.set $c (call $bfile_get (local.get $child)))
      (if (i32.eq (local.get $c) (i32.const 13))
        (then
          (local.set $got (call $bfile_get (local.get $child)))
          (if (i32.eq (local.get $got) (i32.const 10)) (then (return (i32.const 10))))
          (call $bfile_unget (local.get $got) (local.get $child))))
      (return (local.get $c))))
  (if (i32.eqz (i32.and (i32.load offset=4 (local.get $h)) (i32.const 1)))
    (then (return (call $bfile_set_error (local.get $h) (i32.const 8)))))
  (if (i32.and (i32.ne (local.get $kind) (i32.const 1)) (i32.ne (local.get $kind) (i32.const 2)))
    (then (return (call $bfile_set_error (local.get $h) (i32.const 8)))))
  (if (i32.eq (i32.load offset=40 (local.get $h)) (i32.const 2))
    (then
      (call $bfile_flush (local.get $h))
      (if (call $bfile_error (local.get $h)) (then (return (i32.const -1))))
      (i32.store offset=20 (local.get $h) (i32.const 0))
      (i32.store offset=24 (local.get $h) (i32.const 0))))
  (i32.store offset=40 (local.get $h) (i32.const 1))
  (local.set $pos (i32.load offset=20 (local.get $h)))
  (if (i32.ge_u (local.get $pos) (i32.load offset=24 (local.get $h)))
    (then
      (if (i32.eq (local.get $kind) (i32.const 2)) (then (return (i32.const -1))))
      (local.set $got (call $wasi_read (i32.load offset=8 (local.get $h))
        (i32.load offset=12 (local.get $h)) (i32.load offset=16 (local.get $h))))
      (if (i32.lt_s (local.get $got) (i32.const 0))
        (then (return (call $bfile_set_error (local.get $h) (i32.load (i32.const 0x240))))))
      (if (i32.eqz (local.get $got)) (then (return (i32.const -1))))
      (i32.store offset=24 (local.get $h) (local.get $got))
      (local.set $pos (i32.const 0))))
  (local.set $c (i32.load8_u (i32.add (i32.load offset=12 (local.get $h)) (local.get $pos))))
  (i32.store offset=20 (local.get $h) (i32.add (local.get $pos) (i32.const 1)))
  (local.get $c))

;; Modified UTF-8 writes NUL as c0 80. All other code points use the C layer's
;; one-to-four-byte encoding, including its permitted surrogate code points.
(func $bfile_put_utf8 (param $c i32) (param $h i32)
  (local $child i32)
  (local.set $child (i32.load offset=36 (local.get $h)))
  (if (i32.or (i32.lt_s (local.get $c) (i32.const 0)) (i32.ge_u (local.get $c) (i32.const 0x110000)))
    (then (drop (call $bfile_set_error (local.get $h) (i32.const 25))) (return)))
  (if (i32.and (i32.gt_u (local.get $c) (i32.const 0)) (i32.lt_u (local.get $c) (i32.const 128)))
    (then (call $bfile_put (local.get $c) (local.get $child)) (return)))
  (if (i32.lt_u (local.get $c) (i32.const 2048))
    (then (call $bfile_put (i32.or (i32.shr_u (local.get $c) (i32.const 6)) (i32.const 192)) (local.get $child)))
    (else
      (if (i32.lt_u (local.get $c) (i32.const 65536))
        (then (call $bfile_put (i32.or (i32.shr_u (local.get $c) (i32.const 12)) (i32.const 224)) (local.get $child)))
        (else
          (call $bfile_put (i32.or (i32.shr_u (local.get $c) (i32.const 18)) (i32.const 240)) (local.get $child))
          (call $bfile_put (i32.or (i32.and (i32.shr_u (local.get $c) (i32.const 12)) (i32.const 63)) (i32.const 128)) (local.get $child))))
      (call $bfile_put (i32.or (i32.and (i32.shr_u (local.get $c) (i32.const 6)) (i32.const 63)) (i32.const 128)) (local.get $child))))
  (call $bfile_put (i32.or (i32.and (local.get $c) (i32.const 63)) (i32.const 128)) (local.get $child)))

(func $bfile_put (param $c i32) (param $h i32)
  (local $kind i32) (local $pos i32) (local $unread i32) (local $error i32) (local $flags i32)
  (if (i32.eqz (local.get $h)) (then (drop (call $io_errno (i32.const 8))) (return)))
  (local.set $kind (i32.load (local.get $h)))
  (if (i32.eq (local.get $kind) (i32.const 4)) (then (call $bfile_put_utf8 (local.get $c) (local.get $h)) (return)))
  (if (i32.eq (local.get $kind) (i32.const 5))
    (then
      (if (i32.eq (local.get $c) (i32.const 10))
        (then (call $bfile_put (i32.const 13) (i32.load offset=36 (local.get $h)))))
      (call $bfile_put (local.get $c) (i32.load offset=36 (local.get $h)))
      (return)))
  (local.set $flags (i32.load offset=4 (local.get $h)))
  (if (i32.eqz (i32.and (local.get $flags) (i32.const 2)))
    (then (drop (call $bfile_set_error (local.get $h) (i32.const 8))) (return)))
  (if (i32.eq (local.get $kind) (i32.const 3))
    (then
      (local.set $pos (i32.load offset=20 (local.get $h)))
      (call $bfile_reserve (local.get $h) (i32.add (local.get $pos) (i32.const 1)))
      (i32.store8 (i32.add (i32.load offset=12 (local.get $h)) (local.get $pos)) (local.get $c))
      (i32.store offset=20 (local.get $h) (i32.add (local.get $pos) (i32.const 1)))
      (return)))
  (if (i32.ne (local.get $kind) (i32.const 1))
    (then (drop (call $bfile_set_error (local.get $h) (i32.const 8))) (return)))
  (if (i32.eq (i32.load offset=40 (local.get $h)) (i32.const 1))
    (then
      (local.set $unread (i32.sub (i32.load offset=24 (local.get $h)) (i32.load offset=20 (local.get $h))))
      (local.set $unread (i32.add (local.get $unread) (i32.ge_s (i32.load offset=28 (local.get $h)) (i32.const 0))))
      (if (local.get $unread)
        (then
          (local.set $error (call $wasi_fd_seek (i32.load offset=8 (local.get $h))
            (i64.sub (i64.const 0) (i64.extend_i32_u (local.get $unread))) (i32.const 1) (i32.const 0x220)))
          (if (local.get $error)
            (then (drop (call $bfile_set_error (local.get $h) (local.get $error))) (return)))))
      (i32.store offset=20 (local.get $h) (i32.const 0))
      (i32.store offset=24 (local.get $h) (i32.const 0))
      (i32.store offset=28 (local.get $h) (i32.const -1))))
  (i32.store offset=40 (local.get $h) (i32.const 2))
  (local.set $pos (i32.load offset=20 (local.get $h)))
  (if (i32.eq (local.get $pos) (i32.load offset=16 (local.get $h)))
    (then
      (call $bfile_flush (local.get $h))
      (if (call $bfile_error (local.get $h)) (then (return)))
      (local.set $pos (i32.load offset=20 (local.get $h)))))
  (i32.store8 (i32.add (i32.load offset=12 (local.get $h)) (local.get $pos)) (local.get $c))
  (i32.store offset=20 (local.get $h) (i32.add (local.get $pos) (i32.const 1)))
  (if (i32.or (i32.and (local.get $flags) (i32.const 16))
        (i32.and (i32.ne (i32.and (local.get $flags) (i32.const 8)) (i32.const 0)) (i32.eq (local.get $c) (i32.const 10))))
    (then (call $bfile_flush (local.get $h)))))

;; Block operations preserve the C fallback semantics on transducers: readb
;; keeps the low byte of each decoded character, and writeb encodes each byte.
(func $bfile_read (param $p i32) (param $n i32) (param $h i32) (result i32)
  (local $done i32) (local $c i32)
  (block $end
    (loop $next
      (br_if $end (i32.eq (local.get $done) (local.get $n)))
      (local.set $c (call $bfile_get (local.get $h)))
      (br_if $end (i32.lt_s (local.get $c) (i32.const 0)))
      (i32.store8 (i32.add (local.get $p) (local.get $done)) (local.get $c))
      (local.set $done (i32.add (local.get $done) (i32.const 1)))
      (br $next)))
  (local.get $done))

(func $bfile_write (param $p i32) (param $n i32) (param $h i32) (result i32)
  (local $done i32) (local $pos i32)
  (if (i32.eq (i32.load (local.get $h)) (i32.const 3))
    (then
      (local.set $pos (i32.load offset=20 (local.get $h)))
      (if (i32.lt_u (i32.add (local.get $pos) (local.get $n)) (local.get $pos)) (then (call $fail (i32.const 48))))
      (call $bfile_reserve (local.get $h) (i32.add (local.get $pos) (local.get $n)))
      (memory.copy (i32.add (i32.load offset=12 (local.get $h)) (local.get $pos)) (local.get $p) (local.get $n))
      (i32.store offset=20 (local.get $h) (i32.add (local.get $pos) (local.get $n)))
      (return (local.get $n))))
  (block $end
    (loop $next
      (br_if $end (i32.eq (local.get $done) (local.get $n)))
      (call $bfile_put (i32.load8_u (i32.add (local.get $p) (local.get $done))) (local.get $h))
      (br_if $end (call $bfile_error (local.get $h)))
      (local.set $done (i32.add (local.get $done) (i32.const 1)))
      (br $next)))
  (local.get $done))

;; Read the remaining stream for the serialized-program parser. The returned
;; buffer is owned independently of the stream and remains valid after close.
(func $bfile_read_all (param $h i32) (result i32 i32)
  (local $out i32) (local $c i32) (local $p i32) (local $n i32)
  (local.set $out (call $bfile_wr_mem))
  (block $end
    (loop $read
      (local.set $c (call $bfile_get (local.get $h)))
      (br_if $end (i32.lt_s (local.get $c) (i32.const 0)))
      (call $bfile_put (local.get $c) (local.get $out))
      (br $read)))
  (local.set $p (i32.load offset=12 (local.get $out)))
  (local.set $n (i32.load offset=20 (local.get $out)))
  (call $bfile_close (local.get $out))
  (if (call $bfile_error (local.get $h))
    (then (call $blob_free (local.get $p)) (return (i32.const 0) (i32.const 0))))
  (local.get $p) (local.get $n))

;; Read one serialized record without consuming its stream trailer. A closing
;; brace ends a record only between tokens. Quotes and raw byte counts protect
;; embedded braces. Numeric tokens can end immediately before the closing brace.
;; The loader validates the collected record and reports syntax failures.
;; Stop before the trailer, so UTF-8 code points and transducer state remain
;; in the original stream. Reading all bytes and pushing a suffix back would
;; lose decoded values above 255 on a UTF-8 stream.
(func $bfile_read_record (param $h i32) (result i32 i32)
  (local $out i32) (local $c i32) (local $p i32) (local $n i32)
  (local $state i32) (local $remaining i32) (local $digits i32)
  ;; States: 0=between, 1=name, 2=quote, 3=escape, 4=raw size,
  ;; 5=raw bytes, 6=integer or label, 7=array size, 8=tick quote.
  (local.set $out (call $bfile_wr_mem))
  (block $end
    (loop $read
      (local.set $c (call $bfile_get (local.get $h)))
      (br_if $end (i32.lt_s (local.get $c) (i32.const 0)))
      (call $bfile_put (local.get $c) (local.get $out))
      (if (i32.eq (local.get $state) (i32.const 1))
        (then
          (if (call $parse_space (local.get $c)) (then (local.set $state (i32.const 0))))
          (br $read)))
      (if (i32.eq (local.get $state) (i32.const 2))
        (then
          (if (i32.eq (local.get $c) (i32.const 34))
            (then (local.set $state (i32.const 0)))
            (else
              (if (i32.or (i32.eq (local.get $c) (i32.const 92))
                    (i32.or (i32.eq (local.get $c) (i32.const 94))
                      (i32.eq (local.get $c) (i32.const 124))))
                (then (local.set $state (i32.const 3))))))
          (br $read)))
      (if (i32.eq (local.get $state) (i32.const 3))
        (then (local.set $state (i32.const 2)) (br $read)))
      (if (i32.eq (local.get $state) (i32.const 4))
        (then
          (if (i32.le_u (i32.sub (local.get $c) (i32.const 48)) (i32.const 9))
            (then
              (br_if $end (i32.or (i32.gt_u (local.get $remaining) (i32.const 429496729))
                (i32.and (i32.eq (local.get $remaining) (i32.const 429496729))
                  (i32.gt_u (local.get $c) (i32.const 53)))))
              (local.set $remaining (i32.add (i32.mul (local.get $remaining) (i32.const 10))
                (i32.sub (local.get $c) (i32.const 48))))
              (local.set $digits (i32.const 1)))
            (else
              (br_if $end (i32.or (i32.eqz (local.get $digits)) (i32.ne (local.get $c) (i32.const 32))))
              (local.set $state (select (i32.const 5) (i32.const 0) (local.get $remaining)))))
          (br $read)))
      (if (i32.eq (local.get $state) (i32.const 5))
        (then
          (local.set $remaining (i32.sub (local.get $remaining) (i32.const 1)))
          (if (i32.eqz (local.get $remaining)) (then (local.set $state (i32.const 0))))
          (br $read)))
      (if (i32.eq (local.get $state) (i32.const 7))
        (then
          (if (i32.eq (local.get $c) (i32.const 93))
            (then (local.set $state (i32.const 0)))
            (else
              (br_if $end (i32.and (i32.ne (local.get $c) (i32.const 45))
                (i32.gt_u (i32.sub (local.get $c) (i32.const 48)) (i32.const 9))))))
          (br $read)))
      (if (i32.eq (local.get $state) (i32.const 8))
        (then
          (br_if $end (i32.ne (local.get $c) (i32.const 34)))
          (local.set $state (i32.const 2)) (br $read)))
      (if (i32.eq (local.get $state) (i32.const 6))
        (then
          (br_if $read (i32.or (i32.eq (local.get $c) (i32.const 35))
            (i32.or (i32.eq (local.get $c) (i32.const 45))
              (i32.le_u (i32.sub (local.get $c) (i32.const 48)) (i32.const 9)))))
          (local.set $state (i32.const 0))))
      (br_if $read (call $parse_space (local.get $c)))
      (br_if $end (i32.eq (local.get $c) (i32.const 125)))
      (br_if $read (i32.eq (local.get $c) (i32.const 64)))
      (if (i32.eq (local.get $c) (i32.const 34))
        (then (local.set $state (i32.const 2)) (br $read)))
      (if (i32.eq (local.get $c) (i32.const 36))
        (then
          (local.set $state (i32.const 4))
          (local.set $remaining (i32.const 0))
          (local.set $digits (i32.const 0))
          (br $read)))
      (if (i32.eq (local.get $c) (i32.const 33))
        (then (local.set $state (i32.const 8)) (br $read)))
      (if (i32.eq (local.get $c) (i32.const 91))
        (then (local.set $state (i32.const 7)) (br $read)))
      (if (i32.or (i32.eq (local.get $c) (i32.const 35))
            (i32.or (i32.eq (local.get $c) (i32.const 95))
              (i32.eq (local.get $c) (i32.const 58))))
        (then (local.set $state (i32.const 6)) (br $read)))
      (local.set $state (i32.const 1))
      (br $read)))
  (local.set $p (i32.load offset=12 (local.get $out)))
  (local.set $n (i32.load offset=20 (local.get $out)))
  (call $bfile_close (local.get $out))
  (if (call $bfile_error (local.get $h))
    (then (call $blob_free (local.get $p)) (return (i32.const 0) (i32.const 0))))
  (local.get $p) (local.get $n))

;; fopen and add_FILE share one descriptor in this implementation. No other
;; operation exposes a FILE object, so a second wrapper is unnecessary.
(func $io_fopen (param $path i32) (param $mode i32) (result i32)
  (local $flags i32) (local $fd i32) (local $h i32) (local $p i32) (local $c i32)
  (local.set $c (i32.load8_u (local.get $mode)))
  (if (i32.eq (local.get $c) (i32.const 114)) (then (local.set $flags (i32.const 1))))
  (if (i32.eq (local.get $c) (i32.const 119)) (then (local.set $flags (i32.const 26))))
  (if (i32.eq (local.get $c) (i32.const 97)) (then (local.set $flags (i32.const 22))))
  (if (i32.eqz (local.get $flags))
    (then (drop (call $io_errno (i32.const 28))) (return (i32.const 0))))
  (local.set $p (i32.add (local.get $mode) (i32.const 1)))
  (block $done
    (loop $next
      (local.set $c (i32.load8_u (local.get $p)))
      (br_if $done (i32.eqz (local.get $c)))
      (if (i32.eq (local.get $c) (i32.const 43))
        (then (local.set $flags (i32.or (local.get $flags) (i32.const 3)))))
      (local.set $p (i32.add (local.get $p) (i32.const 1)))
      (br $next)))
  (local.set $fd (call $wasi_open (local.get $path) (call $strlen (local.get $path)) (local.get $flags)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (i32.const 0))))
  (local.set $h (call $bfile_fd (local.get $fd) (i32.const 1)))
  (i32.store offset=4 (local.get $h) (i32.or (i32.and (local.get $flags) (i32.const 3)) (i32.const 4)))
  (local.get $h))

;; Preserve errno when a service has no Haskell error return. Errno values are
;; not runtime exception codes. Failure 94 stops without entering Haskell catch.
(func $io_fail (param $error i32) (result i32)
  (drop (call $io_errno (local.get $error)))
  (call $fail (i32.const 94))
  (unreachable))

(func $io_stream_result (param $h i32) (param $result i32) (result i32)
  (local $error i32)
  (local.set $error (call $bfile_error (local.get $h)))
  (if (local.get $error) (then (return (call $io_fail (local.get $error)))))
  (local.get $result))

;; Return a scalar result, without the IO pair. The evaluator supplies World.
;; Zero means an unknown service. Reads retain their error sentinel or count.
;; Keep character operations first: the compiler calls them for source text.
(func $service_io (param $id i32) (param $args i32) (result i32)
  (local $a i32) (local $b i32) (local $c i32) (local $h i32) (local $p i32) (local $n i32)
  (local.set $a (call $value_arg (local.get $args) (i32.const 0)))
  (local.set $b (call $value_arg (local.get $args) (i32.const 1)))
  (local.set $c (call $value_arg (local.get $args) (i32.const 2)))
  (if (i32.eq (local.get $id) (global.get $svc_getb))
    (then
      (local.set $h (call $pval (local.get $a)))
      (return (call $int (call $bfile_get (local.get $h))))))
  (if (i32.eq (local.get $id) (global.get $svc_putb))
    (then
      (local.set $h (call $pval (local.get $b)))
      (call $bfile_put (call $ival (local.get $a)) (local.get $h))
      (return (call $io_stream_result (local.get $h) (call $prim (global.get $T_I))))))
  (if (i32.eq (local.get $id) (global.get $svc_ungetb))
    (then
      (local.set $h (call $pval (local.get $b)))
      (call $bfile_unget (call $ival (local.get $a)) (local.get $h))
      (return (call $io_stream_result (local.get $h) (call $prim (global.get $T_I))))))
  (if (i32.eq (local.get $id) (global.get $svc_readb))
    (then
      (local.set $h (call $pval (local.get $c)))
      (local.set $n (call $ival (local.get $b)))
      (if (i32.lt_s (local.get $n) (i32.const 0)) (then (return (call $io_fail (i32.const 28)))))
      (return (call $int (call $bfile_read (call $pval (local.get $a)) (local.get $n) (local.get $h))))))
  (if (i32.eq (local.get $id) (global.get $svc_writeb))
    (then
      (local.set $h (call $pval (local.get $c)))
      (local.set $n (call $ival (local.get $b)))
      (if (i32.lt_s (local.get $n) (i32.const 0)) (then (return (call $io_fail (i32.const 28)))))
      (return (call $io_stream_result (local.get $h)
        (call $int (call $bfile_write (call $pval (local.get $a)) (local.get $n) (local.get $h)))))))
  (if (i32.eq (local.get $id) (global.get $svc_fopen))
    (then (return (call $ptr (call $io_fopen (call $pval (local.get $a)) (call $pval (local.get $b)))))))
  (if (i32.eq (local.get $id) (global.get $svc_add_FILE))
    (then (return (call $ptr (call $pval (local.get $a))))))
  (if (i32.eq (local.get $id) (global.get $svc_add_fd))
    (then (return (call $ptr (call $bfile_fd (call $ival (local.get $a)) (i32.const 1))))))
  (if (i32.eq (local.get $id) (global.get $svc_add_utf8))
    (then (return (call $ptr (call $bfile_utf8 (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_add_crlf))
    (then (return (call $ptr (call $bfile_crlf (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_flushb))
    (then
      (local.set $h (call $pval (local.get $a)))
      (call $bfile_flush (local.get $h))
      (return (call $io_stream_result (local.get $h) (call $prim (global.get $T_I))))))
  (if (i32.eq (local.get $id) (global.get $svc_closeb))
    (then (call $bfile_close (call $pval (local.get $a))) (return (call $prim (global.get $T_I)))))
  (if (i32.eq (local.get $id) (global.get $svc_addr_closeb))
    (then
      (local.set $n (call $node (global.get $T_FUNPTR)))
      (i32.store offset=4 (local.get $n) (i32.add (global.get $svc_closeb) (i32.const 1)))
      (return (local.get $n))))
  (if (i32.eq (local.get $id) (global.get $svc_openb_wr_mem))
    (then (return (call $ptr (call $bfile_wr_mem)))))
  (if (i32.eq (local.get $id) (global.get $svc_openb_rd_mem))
    (then
      (local.set $n (call $ival (local.get $b)))
      (if (i32.lt_s (local.get $n) (i32.const 0)) (then (return (call $io_fail (i32.const 28)))))
      (return (call $ptr (call $bfile_rd_mem (call $pval (local.get $a)) (local.get $n))))))
  (if (i32.eq (local.get $id) (global.get $svc_get_mem))
    (then
      (local.set $h (call $pval (local.get $a)))
      (if (i32.ne (i32.load (local.get $h)) (i32.const 3)) (then (return (call $io_fail (i32.const 28)))))
      (i32.store (call $pval (local.get $b)) (i32.load offset=12 (local.get $h)))
      (i32.store (call $pval (local.get $c)) (i32.load offset=20 (local.get $h)))
      (return (call $prim (global.get $T_I)))))
  (if (i32.eq (local.get $id) (global.get $svc_getenv))
    (then (return (call $ptr (call $io_getenv (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_setenv))
    (then (return (call $int (call $io_setenv (call $pval (local.get $a)) (call $pval (local.get $b)) (call $ival (local.get $c)))))))
  (if (i32.eq (local.get $id) (global.get $svc_unsetenv))
    (then (return (call $int (call $io_unsetenv (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_get_executable_path))
    (then (return (call $ptr (call $io_copy_cstr (global.get $wasi_progname))))))
  (if (i32.eq (local.get $id) (global.get $svc_opendir))
    (then (return (call $ptr (call $io_opendir (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_readdir))
    (then (return (call $ptr (call $io_readdir (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_c_d_name))
    (then (return (call $ptr (call $pval (local.get $a))))))
  (if (i32.eq (local.get $id) (global.get $svc_closedir))
    (then (return (call $int (call $io_closedir (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_mkdir))
    (then (return (call $int (call $io_path_operation (i32.const 0) (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_remove))
    (then (return (call $int (call $io_path_operation (i32.const 1) (call $pval (local.get $a)))))))
  (if (i32.eq (local.get $id) (global.get $svc_GETTIMEMICRO))
    (then (return (call $int (call $wasi_micros)))))
  (if (i32.eq (local.get $id) (global.get $svc_GETBOOTTIMEMICRO))
    (then (return (call $int (global.get $wasi_boot_micros)))))
  (if (i32.eq (local.get $id) (global.get $svc_GETRAW))
    (then
      (local.set $n (call $wasi_read (i32.const 0) (i32.const 0x218) (i32.const 1)))
      (return (call $int (select (i32.load8_u (i32.const 0x218)) (i32.const -1) (i32.eq (local.get $n) (i32.const 1)))))))
  ;; Preview 1 cannot launch a subprocess or obtain a host temporary filename.
  ;; A NULL system argument asks whether a command processor exists.
  (if (i32.eq (local.get $id) (global.get $svc_system))
    (then
      (if (i32.eqz (call $pval (local.get $a))) (then (return (call $int (i32.const 0)))))
      (return (call $int (call $io_errno (i32.const 52))))))
  (if (i32.eq (local.get $id) (global.get $svc_tmpname))
    (then (drop (call $io_errno (i32.const 52))) (return (call $ptr (i32.const 0)))))
  ;; Compressed caches require real codecs. A missing codec is a fatal error.
  ;; Returning the original stream would corrupt the file format.
  (if (i32.or (i32.eq (local.get $id) (global.get $svc_add_lz77_compressor))
               (i32.eq (local.get $id) (global.get $svc_add_lz77_decompressor)))
    (then (return (call $io_fail (i32.const 52)))))
  (if (i32.or (i32.eq (local.get $id) (global.get $svc_add_lzma_compressor))
               (i32.eq (local.get $id) (global.get $svc_add_lzma_decompressor)))
    (then (return (call $io_fail (i32.const 52)))))
  (if (i32.or (i32.eq (local.get $id) (global.get $svc_add_rle_compressor))
               (i32.eq (local.get $id) (global.get $svc_add_rle_decompressor)))
    (then (return (call $io_fail (i32.const 52)))))
  (if (i32.or (i32.eq (local.get $id) (global.get $svc_add_bwt_compressor))
               (i32.eq (local.get $id) (global.get $svc_add_bwt_decompressor)))
    (then (return (call $io_fail (i32.const 52)))))
  (if (i32.or (i32.eq (local.get $id) (global.get $svc_add_base64_encoder))
               (i32.eq (local.get $id) (global.get $svc_add_base64_decoder)))
    (then (return (call $io_fail (i32.const 52)))))
  (i32.const 0))
