;; Host operations for the standalone runtime. There is no C ABI in this file.
;; The service layer translates the historical bytecode names to these calls.
;;
;; Scratch memory 0x200..0x23f is private to synchronous WASI calls. The errno
;; value is at 0x240. No call in this file invokes evaluation or collection.
;; WASI returns an errno value. Helpers return -1 on failure and retain errno.
;;
;; The roots at 0x3020, 0x3024, and 0x3028 retain the argv table, environment
;; table, and preopen list. 0x302c retains the selected program-name pointer.
;; Pointer tables use mask -1. Preopen records are {fd, length, name, next}.
;; The pointer mask for a preopen record is 12, for words 2 and 3.
;;
;; Preview 1 reference: WebAssembly/WASI commit
;; a2b96e81c0586125cc4dc79a5be0b78d9a059925, legacy/preview1/docs.md.
(global $wasi_argc (mut i32) (i32.const 0))
(global $wasi_argv (mut i32) (i32.const 0))
(global $wasi_envc (mut i32) (i32.const 0))
(global $wasi_envcap (mut i32) (i32.const 0))
(global $wasi_env (mut i32) (i32.const 0))
(global $wasi_preopens (mut i32) (i32.const 0))
(global $wasi_progname (mut i32) (i32.const 0))
(global $wasi_stdin (mut i32) (i32.const 0))
(global $wasi_stdout (mut i32) (i32.const 0))
(global $wasi_stderr (mut i32) (i32.const 0))
(global $wasi_boot_micros (mut i32) (i32.const 0))
(data (i32.const 0xd000) ".\00mhs\00")

;; Copy a complete NUL-terminated byte string into an owned payload block.
(func $io_copy_cstr (param $p i32) (result i32)
  (local $n i32) (local $q i32)
  (local.set $n (i32.add (call $strlen (local.get $p)) (i32.const 1)))
  (local.set $q (call $blob_alloc (local.get $n) (i32.const 0)))
  (memory.copy (local.get $q) (local.get $p) (local.get $n))
  (local.get $q))

;; Preserve the WASI errno number. The errno service uses the same numbering.
(func $io_errno (param $error i32) (result i32)
  (i32.store (i32.const 0x240) (local.get $error))
  (i32.const -1))

;; Return the wrapped 32-bit microsecond clock used by MicroHs TimeMilli.
(func $wasi_micros (result i32)
  (local $error i32)
  (local.set $error (call $wasi_clock_time_get (i32.const 0) (i64.const 1000) (i32.const 0x220)))
  (if (local.get $error) (then (return (call $io_errno (local.get $error)))))
  (i32.wrap_i64 (i64.div_u (i64.load (i32.const 0x220)) (i64.const 1000))))

;; One read can return fewer bytes than requested. Zero means end of file.
(func $wasi_read (param $fd i32) (param $p i32) (param $n i32) (result i32)
  (local $error i32)
  (i32.store (i32.const 0x200) (local.get $p))
  (i32.store (i32.const 0x204) (local.get $n))
  (local.set $error (call $wasi_fd_read (local.get $fd) (i32.const 0x200) (i32.const 1) (i32.const 0x208)))
  (if (local.get $error) (then (return (call $io_errno (local.get $error)))))
  (i32.load (i32.const 0x208)))

;; Complete a write unless the host reports an error. A zero-length host write
;; cannot make progress and is reported as EIO instead of looping forever.
(func $wasi_write_all (param $fd i32) (param $p i32) (param $n i32) (result i32)
  (local $done i32) (local $error i32) (local $wrote i32)
  (block $end
    (loop $next
      (br_if $end (i32.eq (local.get $done) (local.get $n)))
      (i32.store (i32.const 0x200) (i32.add (local.get $p) (local.get $done)))
      (i32.store (i32.const 0x204) (i32.sub (local.get $n) (local.get $done)))
      (local.set $error (call $wasi_fd_write (local.get $fd) (i32.const 0x200) (i32.const 1) (i32.const 0x208)))
      (if (local.get $error) (then (drop (call $io_errno (local.get $error))) (br $end)))
      (local.set $wrote (i32.load (i32.const 0x208)))
      (if (i32.eqz (local.get $wrote)) (then (drop (call $io_errno (i32.const 29))) (br $end)))
      (local.set $done (i32.add (local.get $done) (local.get $wrote)))
      (br $next)))
  (local.get $done))

;; Match one preopen name. Return the path bytes to skip, or -1 for no match.
(func $wasi_prefix (param $p i32) (param $n i32) (param $name i32) (param $len i32) (result i32)
  (local $i i32)
  (if (i32.and (i32.eq (local.get $len) (i32.const 1))
               (i32.eq (i32.load8_u (local.get $name)) (i32.const 46)))
    (then (return (select (i32.const 0) (i32.const -1)
      (i32.or (i32.eqz (local.get $n)) (i32.ne (i32.load8_u (local.get $p)) (i32.const 47)))))))
  (if (i32.and (i32.eq (local.get $len) (i32.const 1))
               (i32.eq (i32.load8_u (local.get $name)) (i32.const 47)))
    (then (return (i32.and (i32.ne (local.get $n) (i32.const 0))
      (i32.eq (i32.load8_u (local.get $p)) (i32.const 47))))))
  (if (i32.lt_u (local.get $n) (local.get $len)) (then (return (i32.const -1))))
  (block $matched
    (loop $compare
      (br_if $matched (i32.eq (local.get $i) (local.get $len)))
      (if (i32.ne (i32.load8_u (i32.add (local.get $p) (local.get $i)))
                  (i32.load8_u (i32.add (local.get $name) (local.get $i))))
        (then (return (i32.const -1))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $compare)))
  (if (i32.eq (local.get $n) (local.get $len)) (then (return (local.get $len))))
  (if (i32.eq (i32.load8_u (i32.add (local.get $p) (local.get $len))) (i32.const 47))
    (then (return (i32.add (local.get $len) (i32.const 1)))))
  (i32.const -1))

;; Choose the longest matching guest preopen prefix. The host enforces path
;; traversal and symlink capabilities. Relative paths use the '.' preopen, or
;; the '/' preopen when the host provides a virtual root. Return fd, path, len.
(func $wasi_resolve (param $p i32) (param $n i32) (result i32 i32 i32)
  (local $entry i32) (local $name i32) (local $len i32) (local $skip i32)
  (local $score i32) (local $best i32) (local $fd i32) (local $outp i32) (local $outn i32)
  (local.set $fd (i32.const -1))
  (local.set $best (i32.const -1))
  (block $stripped
    (loop $strip
      (br_if $stripped (i32.lt_u (local.get $n) (i32.const 2)))
      (br_if $stripped (i32.ne (i32.load16_u (local.get $p)) (i32.const 0x2f2e)))
      (local.set $p (i32.add (local.get $p) (i32.const 2)))
      (local.set $n (i32.sub (local.get $n) (i32.const 2)))
      (br $strip)))
  (local.set $entry (global.get $wasi_preopens))
  (block $end
    (loop $next
      (br_if $end (i32.eqz (local.get $entry)))
      (local.set $name (i32.load offset=8 (local.get $entry)))
      (local.set $len (i32.load offset=4 (local.get $entry)))
      (local.set $skip (call $wasi_prefix (local.get $p) (local.get $n) (local.get $name) (local.get $len)))
      (local.set $score (local.get $len))
      (if (i32.and (i32.eq (local.get $len) (i32.const 1))
                   (i32.eq (i32.load8_u (local.get $name)) (i32.const 46)))
        (then (local.set $score (i32.const 0))))
      (if (i32.and (i32.ge_s (local.get $skip) (i32.const 0))
                   (i32.gt_s (local.get $score) (local.get $best)))
        (then
          (local.set $best (local.get $score))
          (local.set $fd (i32.load (local.get $entry)))
          (local.set $outp (i32.add (local.get $p) (local.get $skip)))
          (local.set $outn (i32.sub (local.get $n) (local.get $skip)))))
      (local.set $entry (i32.load offset=12 (local.get $entry)))
      (br $next)))
  (if (i32.lt_s (local.get $fd) (i32.const 0))
    (then (drop (call $io_errno (i32.const 76)))))
  (if (i32.eqz (local.get $outn))
    (then (local.set $outp (i32.const 0xd000)) (local.set $outn (i32.const 1))))
  (local.get $fd) (local.get $outp) (local.get $outn))

;; Internal open flags: READ=1, WRITE=2, APPEND=4, TRUNCATE=8, CREATE=16,
;; DIRECTORY=32. Request only rights required by the resulting descriptor.
(func $wasi_open (param $p i32) (param $n i32) (param $flags i32) (result i32)
  (local $fd i32) (local $oflags i32) (local $error i32) (local $rights i64)
  (call $wasi_resolve (local.get $p) (local.get $n))
  (local.set $n) (local.set $p) (local.set $fd)
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (i32.const -1))))
  (local.set $rights (i64.const 2097152))
  (if (i32.and (local.get $flags) (i32.const 32))
    (then (local.set $oflags (i32.const 2))
      (local.set $rights (i64.or (local.get $rights) (i64.const 16384))))
    (else
      ;; Seek, tell, and scheduler readiness are available on regular files.
      (local.set $rights (i64.or (local.get $rights) (i64.const 134217764)))
      (if (i32.and (local.get $flags) (i32.const 1))
        (then (local.set $rights (i64.or (local.get $rights) (i64.const 2)))))
      (if (i32.and (local.get $flags) (i32.const 2))
        (then (local.set $rights (i64.or (local.get $rights) (i64.const 4194368)))))
      (if (i32.and (local.get $flags) (i32.const 16))
        (then (local.set $oflags (i32.or (local.get $oflags) (i32.const 1)))))
      (if (i32.and (local.get $flags) (i32.const 8))
        (then (local.set $oflags (i32.or (local.get $oflags) (i32.const 8)))))))
  (local.set $error (call $wasi_path_open
    (local.get $fd) (i32.const 1) (local.get $p) (local.get $n) (local.get $oflags)
    (local.get $rights) (i64.const 0)
    (i32.ne (i32.and (local.get $flags) (i32.const 4)) (i32.const 0)) (i32.const 0x208)))
  (if (local.get $error) (then (return (call $io_errno (local.get $error)))))
  (i32.load (i32.const 0x208)))

;; Read a complete file into an owned blob. Empty files still return a nonzero
;; allocation. Failure returns (0, 0) and preserves errno for the caller.
(func $wasi_read_file (param $p i32) (param $n i32) (result i32 i32)
  (local $fd i32) (local $buf i32) (local $cap i32) (local $used i32)
  (local $next i32) (local $got i32)
  (local.set $fd (call $wasi_open (local.get $p) (local.get $n) (i32.const 1)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (i32.const 0) (i32.const 0))))
  (local.set $cap (i32.const 16384))
  (local.set $buf (call $blob_alloc (local.get $cap) (i32.const 0)))
  (block $end
    (loop $read
      (if (i32.eq (local.get $used) (local.get $cap))
        (then
          (local.set $cap (i32.shl (local.get $cap) (i32.const 1)))
          (local.set $next (call $blob_alloc (local.get $cap) (i32.const 0)))
          (memory.copy (local.get $next) (local.get $buf) (local.get $used))
          (call $blob_free (local.get $buf))
          (local.set $buf (local.get $next))))
      (local.set $got (call $wasi_read (local.get $fd)
        (i32.add (local.get $buf) (local.get $used)) (i32.sub (local.get $cap) (local.get $used))))
      (if (i32.lt_s (local.get $got) (i32.const 0))
        (then
          (drop (call $wasi_fd_close (local.get $fd)))
          (call $blob_free (local.get $buf))
          (return (i32.const 0) (i32.const 0))))
      (br_if $end (i32.eqz (local.get $got)))
      (local.set $used (i32.add (local.get $used) (local.get $got)))
      (br $read)))
  (drop (call $wasi_fd_close (local.get $fd)))
  (local.get $buf) (local.get $used))

;; Read argv, environment, and the contiguous preopen descriptors supplied by
;; Wasmtime and the browser shim. Call this after the heap is initialized.
(func $wasi_init
  (local $error i32) (local $size i32) (local $buf i32) (local $fd i32)
  (local $name i32) (local $entry i32)
  (local.set $error (call $wasi_args_sizes_get (i32.const 0x200) (i32.const 0x204)))
  (if (local.get $error) (then (call $fail (local.get $error))))
  (global.set $wasi_argc (i32.load (i32.const 0x200)))
  (local.set $size (i32.load (i32.const 0x204)))
  (global.set $wasi_argv (call $blob_alloc
    (i32.shl (i32.add (global.get $wasi_argc) (i32.const 1)) (i32.const 2)) (i32.const -1)))
  (memory.fill (global.get $wasi_argv) (i32.const 0)
    (i32.shl (i32.add (global.get $wasi_argc) (i32.const 1)) (i32.const 2)))
  (local.set $buf (call $blob_alloc (i32.add (local.get $size) (i32.const 1)) (i32.const 0)))
  (local.set $error (call $wasi_args_get (global.get $wasi_argv) (local.get $buf)))
  (if (local.get $error) (then (call $fail (local.get $error))))
  (i32.store (i32.const 0x3020) (global.get $wasi_argv))
  (global.set $wasi_progname
    (if (result i32) (global.get $wasi_argc)
      (then (i32.load (global.get $wasi_argv))) (else (i32.const 0xd002))))
  (i32.store (i32.const 0x302c) (global.get $wasi_progname))
  (local.set $error (call $wasi_environ_sizes_get (i32.const 0x200) (i32.const 0x204)))
  (if (local.get $error) (then (call $fail (local.get $error))))
  (global.set $wasi_envc (i32.load (i32.const 0x200)))
  (local.set $size (i32.load (i32.const 0x204)))
  (global.set $wasi_envcap (i32.add (global.get $wasi_envc) (i32.const 8)))
  (global.set $wasi_env (call $blob_alloc (i32.shl (global.get $wasi_envcap) (i32.const 2)) (i32.const -1)))
  (memory.fill (global.get $wasi_env) (i32.const 0) (i32.shl (global.get $wasi_envcap) (i32.const 2)))
  (local.set $buf (call $blob_alloc (i32.add (local.get $size) (i32.const 1)) (i32.const 0)))
  (local.set $error (call $wasi_environ_get (global.get $wasi_env) (local.get $buf)))
  (if (local.get $error) (then (call $fail (local.get $error))))
  (i32.store (i32.const 0x3024) (global.get $wasi_env))
  (local.set $fd (i32.const 3))
  (block $done
    (loop $preopen
      (local.set $error (call $wasi_fd_prestat_get (local.get $fd) (i32.const 0x200)))
      (br_if $done (i32.eq (local.get $error) (i32.const 8)))
      (if (i32.eqz (local.get $error))
        (then
          (local.set $size (i32.load (i32.const 0x204)))
          (local.set $name (call $blob_alloc (i32.add (local.get $size) (i32.const 1)) (i32.const 0)))
          (local.set $error (call $wasi_fd_prestat_dir_name (local.get $fd) (local.get $name) (local.get $size)))
          (if (local.get $error) (then (call $fail (local.get $error))))
          (i32.store8 (i32.add (local.get $name) (local.get $size)) (i32.const 0))
          (local.set $entry (call $blob_alloc (i32.const 16) (i32.const 12)))
          (i32.store (local.get $entry) (local.get $fd))
          (i32.store offset=4 (local.get $entry) (local.get $size))
          (i32.store offset=8 (local.get $entry) (local.get $name))
          (i32.store offset=12 (local.get $entry) (global.get $wasi_preopens))
          (global.set $wasi_preopens (local.get $entry))))
      (local.set $fd (i32.add (local.get $fd) (i32.const 1)))
      (br $preopen)))
  (i32.store (i32.const 0x3028) (global.get $wasi_preopens))
  (global.set $wasi_stdin (call $bfile_utf8 (call $bfile_fd (i32.const 0) (i32.const 0))))
  (global.set $wasi_stdout (call $bfile_utf8 (call $bfile_fd (i32.const 1) (i32.const 0))))
  (global.set $wasi_stderr (call $bfile_utf8 (call $bfile_fd (i32.const 2) (i32.const 0))))
  (i32.store (i32.const 0x3010) (call $io_foreign_pointer (global.get $wasi_stdin)))
  (i32.store (i32.const 0x3014) (call $io_foreign_pointer (global.get $wasi_stdout)))
  (i32.store (i32.const 0x3018) (call $io_foreign_pointer (global.get $wasi_stderr)))
  (global.set $wasi_boot_micros (call $wasi_micros)))

;; C mkStringC exposes argument bytes without UTF-8 decoding. bsunpack retains
;; that behavior lazily. The result is the one-element array used by getArgRef.
(func $wasi_args_setup (param $skip i32) (result i32)
  (local $i i32) (local $list i32) (local $p i32) (local $desc i32) (local $node i32)
  (if (i32.gt_u (local.get $skip) (global.get $wasi_argc)) (then (call $fail (i32.const 28))))
  (global.set $wasi_progname
    (if (result i32) (i32.lt_u (local.get $skip) (global.get $wasi_argc))
      (then (i32.load (i32.add (global.get $wasi_argv) (i32.shl (local.get $skip) (i32.const 2)))))
      (else (i32.const 0xd002))))
  (i32.store (i32.const 0x302c) (global.get $wasi_progname))
  (local.set $i (global.get $wasi_argc))
  (local.set $list (call $prim (global.get $T_K)))
  (block $done
    (loop $argument
      (br_if $done (i32.eq (local.get $i) (local.get $skip)))
      (local.set $i (i32.sub (local.get $i) (i32.const 1)))
      (local.set $p (i32.load (i32.add (global.get $wasi_argv) (i32.shl (local.get $i) (i32.const 2)))))
      (local.set $list (call $ap
        (call $ap (call $prim (global.get $T_O))
          (call $ap (call $prim (global.get $T_BSUNPACK)) (call $bytes_view (local.get $p) (call $strlen (local.get $p)))))
        (local.get $list)))
      (br $argument)))
  (local.set $desc (call $blob_alloc (i32.const 8) (i32.const 2)))
  (i32.store (local.get $desc) (i32.const 1))
  (i32.store offset=4 (local.get $desc) (local.get $list))
  (local.set $node (call $node (global.get $T_ARR)))
  (i32.store offset=4 (local.get $node) (local.get $desc))
  (local.get $node))

(func $wasi_flush
  (call $bfile_flush (global.get $wasi_stdout))
  (call $bfile_flush (global.get $wasi_stderr)))

;; Environment changes belong to this WASI instance. Preview 1 has no host
;; setenv operation. A table entry points to an owned or retained KEY=value
;; byte string. Lookup returns the borrowed value pointer, as getenv does.
(func $io_env_find (param $name i32) (param $length i32) (result i32)
  (local $i i32) (local $j i32) (local $entry i32)
  (block $missing
    (loop $next
      (br_if $missing (i32.eq (local.get $i) (global.get $wasi_envc)))
      (local.set $entry (i32.load (i32.add (global.get $wasi_env) (i32.shl (local.get $i) (i32.const 2)))))
      (local.set $j (i32.const 0))
      (block $different
        (loop $compare
          (br_if $different (i32.eq (local.get $j) (local.get $length)))
          (br_if $different (i32.ne
            (i32.load8_u (i32.add (local.get $entry) (local.get $j)))
            (i32.load8_u (i32.add (local.get $name) (local.get $j)))))
          (local.set $j (i32.add (local.get $j) (i32.const 1)))
          (br $compare)))
      (if (i32.and (i32.eq (local.get $j) (local.get $length))
                   (i32.eq (i32.load8_u (i32.add (local.get $entry) (local.get $j))) (i32.const 61)))
        (then (return (local.get $i))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $next)))
  (i32.const -1))

(func $io_getenv (param $name i32) (result i32)
  (local $length i32) (local $i i32)
  (local.set $length (call $strlen (local.get $name)))
  (local.set $i (call $io_env_find (local.get $name) (local.get $length)))
  (if (i32.lt_s (local.get $i) (i32.const 0)) (then (return (i32.const 0))))
  (i32.add (i32.load (i32.add (global.get $wasi_env) (i32.shl (local.get $i) (i32.const 2))))
    (i32.add (local.get $length) (i32.const 1))))

(func $io_env_valid (param $name i32) (param $length i32) (result i32)
  (local $i i32)
  (if (i32.eqz (local.get $length)) (then (return (i32.const 0))))
  (block $valid
    (loop $next
      (br_if $valid (i32.eq (local.get $i) (local.get $length)))
      (if (i32.eq (i32.load8_u (i32.add (local.get $name) (local.get $i))) (i32.const 61))
        (then (return (i32.const 0))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $next)))
  (i32.const 1))

(func $io_setenv (param $name i32) (param $value i32) (param $overwrite i32) (result i32)
  (local $length i32) (local $vlength i32) (local $i i32) (local $entry i32) (local $table i32)
  (local.set $length (call $strlen (local.get $name)))
  (if (i32.eqz (call $io_env_valid (local.get $name) (local.get $length)))
    (then (return (call $io_errno (i32.const 28)))))
  (local.set $i (call $io_env_find (local.get $name) (local.get $length)))
  (if (i32.and (i32.ge_s (local.get $i) (i32.const 0)) (i32.eqz (local.get $overwrite)))
    (then (return (i32.const 0))))
  (if (i32.lt_s (local.get $i) (i32.const 0))
    (then
      (if (i32.eq (global.get $wasi_envc) (global.get $wasi_envcap))
        (then
          (global.set $wasi_envcap (i32.shl (global.get $wasi_envcap) (i32.const 1)))
          (local.set $table (call $blob_alloc (i32.shl (global.get $wasi_envcap) (i32.const 2)) (i32.const -1)))
          (memory.fill (local.get $table) (i32.const 0) (i32.shl (global.get $wasi_envcap) (i32.const 2)))
          (memory.copy (local.get $table) (global.get $wasi_env) (i32.shl (global.get $wasi_envc) (i32.const 2)))
          (call $blob_free (global.get $wasi_env))
          (global.set $wasi_env (local.get $table))
          (i32.store (i32.const 0x3024) (local.get $table))))
      (local.set $i (global.get $wasi_envc))
      (global.set $wasi_envc (i32.add (global.get $wasi_envc) (i32.const 1)))))
  (local.set $vlength (call $strlen (local.get $value)))
  (local.set $entry (call $blob_alloc (i32.add (i32.add (local.get $length) (local.get $vlength)) (i32.const 2)) (i32.const 0)))
  (memory.copy (local.get $entry) (local.get $name) (local.get $length))
  (i32.store8 (i32.add (local.get $entry) (local.get $length)) (i32.const 61))
  (memory.copy (i32.add (local.get $entry) (i32.add (local.get $length) (i32.const 1)))
    (local.get $value) (i32.add (local.get $vlength) (i32.const 1)))
  (i32.store (i32.add (global.get $wasi_env) (i32.shl (local.get $i) (i32.const 2))) (local.get $entry))
  (i32.const 0))

(func $io_unsetenv (param $name i32) (result i32)
  (local $length i32) (local $i i32) (local $p i32)
  (local.set $length (call $strlen (local.get $name)))
  (if (i32.eqz (call $io_env_valid (local.get $name) (local.get $length)))
    (then (return (call $io_errno (i32.const 28)))))
  (local.set $i (call $io_env_find (local.get $name) (local.get $length)))
  (if (i32.ge_s (local.get $i) (i32.const 0))
    (then
      (global.set $wasi_envc (i32.sub (global.get $wasi_envc) (i32.const 1)))
      (local.set $p (i32.add (global.get $wasi_env) (i32.shl (local.get $i) (i32.const 2))))
      (memory.copy (local.get $p) (i32.add (local.get $p) (i32.const 4))
        (i32.shl (i32.sub (global.get $wasi_envc) (local.get $i)) (i32.const 2)))
      (i32.store (i32.add (global.get $wasi_env) (i32.shl (global.get $wasi_envc) (i32.const 2))) (i32.const 0))))
  (i32.const 0))

;; Open-directory handles use the common stream record. Their buffers contain
;; packed WASI dirent records: next-cookie at 0, inode at 8, name length at 16,
;; type at 20, and name bytes at 24. A returned name remains valid until the next
;; readdir or close. c_d_name is therefore an identity operation in this runtime.
(func $io_opendir (param $path i32) (result i32)
  (local $fd i32) (local $h i32)
  (local.set $fd (call $wasi_open (local.get $path) (call $strlen (local.get $path)) (i32.const 32)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (i32.const 0))))
  (local.set $h (call $bfile_new (i32.const 6)))
  (i32.store offset=4 (local.get $h) (i32.const 4))
  (i32.store offset=8 (local.get $h) (local.get $fd))
  (i32.store offset=12 (local.get $h) (call $blob_alloc (i32.const 4096) (i32.const 0)))
  (i32.store offset=16 (local.get $h) (i32.const 4096))
  (local.get $h))

(func $io_readdir (param $h i32) (result i32)
  (local $pos i32) (local $end i32) (local $cap i32) (local $buf i32)
  (local $entry i32) (local $length i32) (local $name i32) (local $error i32)
  (loop $next
    (local.set $pos (i32.load offset=20 (local.get $h)))
    (local.set $end (i32.load offset=24 (local.get $h)))
    (local.set $cap (i32.load offset=16 (local.get $h)))
    (local.set $buf (i32.load offset=12 (local.get $h)))
    (if (i32.le_u (i32.add (local.get $pos) (i32.const 24)) (local.get $end))
      (then
        (local.set $entry (i32.add (local.get $buf) (local.get $pos)))
        (local.set $length (i32.load offset=16 (local.get $entry)))
        (if (i32.le_u (i32.add (i32.add (local.get $pos) (i32.const 24)) (local.get $length)) (local.get $end))
          (then
            (if (i32.load offset=52 (local.get $h)) (then (call $blob_free (i32.load offset=52 (local.get $h)))))
            (local.set $name (call $blob_alloc (i32.add (local.get $length) (i32.const 1)) (i32.const 0)))
            (memory.copy (local.get $name) (i32.add (local.get $entry) (i32.const 24)) (local.get $length))
            (i32.store8 (i32.add (local.get $name) (local.get $length)) (i32.const 0))
            (i32.store offset=52 (local.get $h) (local.get $name))
            (i64.store offset=44 align=4 (local.get $h) (i64.load align=4 (local.get $entry)))
            (i32.store offset=20 (local.get $h) (i32.add (i32.add (local.get $pos) (i32.const 24)) (local.get $length)))
            (return (local.get $name))))
        (if (i32.gt_u (i32.add (local.get $length) (i32.const 24)) (local.get $cap))
          (then
            (loop $grow
              (if (i32.ge_u (local.get $cap) (i32.const 0x40000000)) (then (call $fail (i32.const 48))))
              (local.set $cap (i32.shl (local.get $cap) (i32.const 1)))
              (br_if $grow (i32.gt_u (i32.add (local.get $length) (i32.const 24)) (local.get $cap))))
            (call $blob_free (local.get $buf))
            (local.set $buf (call $blob_alloc (local.get $cap) (i32.const 0)))
            (i32.store offset=12 (local.get $h) (local.get $buf))
            (i32.store offset=16 (local.get $h) (local.get $cap))))))
    ;; Refill from the last complete entry's cookie. A partial final record
    ;; must be retried; advancing its cookie would silently omit that entry.
    (local.set $error (call $wasi_fd_readdir (i32.load offset=8 (local.get $h))
      (local.get $buf) (local.get $cap) (i64.load offset=44 align=4 (local.get $h)) (i32.const 0x208)))
    (if (local.get $error) (then (drop (call $io_errno (local.get $error))) (return (i32.const 0))))
    (local.set $end (i32.load (i32.const 0x208)))
    (if (i32.eqz (local.get $end)) (then (return (i32.const 0))))
    (if (i32.lt_u (local.get $end) (i32.const 24))
      (then (drop (call $io_errno (i32.const 29))) (return (i32.const 0))))
    (i32.store offset=20 (local.get $h) (i32.const 0))
    (i32.store offset=24 (local.get $h) (local.get $end))
    (br $next))
  (i32.const 0))

(func $io_closedir (param $h i32) (result i32)
  (local $error i32)
  (local.set $error (call $wasi_fd_close (i32.load offset=8 (local.get $h))))
  (i32.store offset=4 (local.get $h) (i32.const 0))
  (call $bfile_close (local.get $h))
  (if (local.get $error) (then (return (call $io_errno (local.get $error)))))
  (i32.const 0))

;; Operation 0 creates a directory. Operation 1 implements remove: unlink a
;; file, or remove a directory if the host reports EISDIR. WASI chooses modes.
(func $io_path_operation (param $op i32) (param $p i32) (result i32)
  (local $fd i32) (local $n i32) (local $error i32)
  (call $wasi_resolve (local.get $p) (call $strlen (local.get $p)))
  (local.set $n) (local.set $p) (local.set $fd)
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.eqz (local.get $op))
    (then (local.set $error (call $wasi_path_create_directory (local.get $fd) (local.get $p) (local.get $n))))
    (else
      (local.set $error (call $wasi_path_unlink_file (local.get $fd) (local.get $p) (local.get $n)))
      (if (i32.eq (local.get $error) (i32.const 31))
        (then (local.set $error (call $wasi_path_remove_directory (local.get $fd) (local.get $p) (local.get $n)))))))
  (if (local.get $error) (then (return (call $io_errno (local.get $error)))))
  (i32.const 0))
