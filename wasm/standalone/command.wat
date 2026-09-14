;; WASI command entry point and host-callable initialization.
;; Usage: mhs-standalone.wasm INPUT.comb -- PROGRAM [ARG ...]
;; The command loads bytecode, constructs argv, and evaluates main World.
;; This file supplies no language evaluation rules and no C ABI wrappers.

(data (i32.const 0xc100) "WAT runtime failure \00 tag \00 service \00\0a\00")
(data (i32.const 0xc160) "usage: mhs-standalone.wasm INPUT.comb -- PROGRAM [ARG ...]\0a\00")


;; Runtime exception messages match Control.Exception.Internal.rtsExn.
(data (i32.const 0xc200) "\30\c2\00\00\3f\c2\00\00\4d\c2\00\00\5b\c2\00\00\6a\c2\00\00\77\c2\00\00\84\c2\00\00\90\c2\00\00\a4\c2\00\00\ae\c2\00\00")
(data (i32.const 0xc230) "\73\74\61\63\6b\20\6f\76\65\72\66\6c\6f\77\00\68\65\61\70\20\6f\76\65\72\66\6c\6f\77\00\74\68\72\65\61\64\20\6b\69\6c\6c\65\64\00\75\73\65\72\20\69\6e\74\65\72\72\75\70\74\00\44\69\76\69\64\65\42\79\5a\65\72\6f\00\62\6c\6f\63\6b\65\64\20\4d\56\61\72\00\62\6c\6f\63\6b\65\64\20\53\54\4d\00\61\72\69\74\68\6d\65\74\69\63\20\6f\76\65\72\66\6c\6f\77\00\53\65\72\69\61\6c\69\7a\65\00\44\65\73\65\72\69\61\6c\69\7a\65\00\75\6e\6b\6e\6f\77\6e\20\72\75\6e\74\69\6d\65\20\65\78\63\65\70\74\69\6f\6e\00\65\78\63\65\70\74\69\6f\6e\20\64\69\73\70\6c\61\79\20\66\61\69\6c\65\64\00\45\78\69\74\53\75\63\63\65\73\73\00\45\78\69\74\46\61\69\6c\75\72\65\20\00\3a\20\75\6e\63\61\75\67\68\74\20\65\78\63\65\70\74\69\6f\6e\3a\20\00")
(global $command_unknown i32 (i32.const 49850))
(global $command_display_failed i32 (i32.const 49876))
(global $command_exit_success i32 (i32.const 49901))
(global $command_exit_failure i32 (i32.const 49913))
(global $command_uncaught_prefix i32 (i32.const 49926))

;; Write an unsigned decimal diagnostic using a fixed scratch buffer.
;; WASI's iovec scratch is separate from 0x280..0x2af used here.
(func $write_u32 (param $fd i32) (param $value i32)
  (local $end i32) (local $p i32)
  (local.set $end (i32.const 0x2a0))
  (local.set $p (local.get $end))
  (loop $digit
    (local.set $p (i32.sub (local.get $p) (i32.const 1)))
    (i32.store8 (local.get $p)
      (i32.add (i32.rem_u (local.get $value) (i32.const 10)) (i32.const 48)))
    (local.set $value (i32.div_u (local.get $value) (i32.const 10)))
    (br_if $digit (local.get $value)))
  (drop (call $wasi_write_all (local.get $fd) (local.get $p)
    (i32.sub (local.get $end) (local.get $p)))))

;; Fatal structural failures are separate from catchable Haskell exceptions.
;; Cache diagnostics before writing because host-call helpers use scratch.
(func $fail (param $code i32)
  (local $tag i32) (local $service i32)
  (local.set $tag (i32.load (i32.const 0x2f0)))
  (local.set $service (i32.load (i32.const 0x2f4)))
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc100) (i32.const 20)))
  (call $write_u32 (i32.const 2) (local.get $code))
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc115) (i32.const 5)))
  (call $write_u32 (i32.const 2) (local.get $tag))
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc11b) (i32.const 9)))
  (call $write_u32 (i32.const 2) (local.get $service))
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc125) (i32.const 1)))
  (call $wasi_proc_exit (i32.const 1))
  (unreachable))

(func $initialize (export "initialize")
  (call $heap_init)
  (global.set $reduction_count (i64.const 0))
  (i32.store (i32.const 0x3000) (call $int (i32.const 99999)))
  (call $wasi_init)
  ;; Standard-handle primitives are permanent values after initialization.
  (memory.copy (call $prim (global.get $T_IO_STDIN))
    (i32.load (i32.const 0x3010)) (i32.const 8))
  (memory.copy (call $prim (global.get $T_IO_STDOUT))
    (i32.load (i32.const 0x3014)) (i32.const 8))
  (memory.copy (call $prim (global.get $T_IO_STDERR))
    (i32.load (i32.const 0x3018)) (i32.const 8)))

;; Collect a Haskell String through sequential outer evaluations. This helper
;; runs only after $execute has returned. It never invokes evaluation from a
;; primitive or from an active evaluator frame. Roots 0x3064 and 0x3068 retain
;; the current list and mutable byte-string builder across each safe point.
(func $collect_string (param $list i32) (result i32)
  (local $builder i32) (local $node i32) (local $fun i32)
  (local $head i32) (local $value i32) (local $result i32)
  (i32.store (i32.const 0x3064) (local.get $list))
  (local.set $builder (call $bytes_view (call $blob_alloc (i32.const 256) (i32.const 0)) (i32.const 0)))
  (i32.store offset=8 (call $bval (local.get $builder)) (i32.const 256))
  (i32.store (i32.const 0x3068) (local.get $builder))
  (block $failed
    (loop $characters
      (local.set $node (call $execute (i32.load (i32.const 0x3064))))
      (br_if $failed (i32.eqz (local.get $node)))
      (local.set $node (call $resolve (local.get $node)))
      (if (i32.eq (call $tag (local.get $node)) (global.get $T_K))
        (then
          (local.set $result (i32.load (i32.const 0x3068)))
          (i32.store offset=8 (call $bval (local.get $result)) (i32.const 0))
          (br $failed)))
      (br_if $failed (i32.ne (i32.and (i32.load (local.get $node)) (i32.const 3)) (i32.const 0)))
      (local.set $fun (call $resolve (i32.load (local.get $node))))
      (br_if $failed (i32.ne (i32.and (i32.load (local.get $fun)) (i32.const 3)) (i32.const 0)))
      (br_if $failed (i32.ne (call $tag (call $resolve (i32.load (local.get $fun)))) (global.get $T_O)))
      (i32.store (i32.const 0x3064) (local.get $node))
      (local.set $head (call $execute (i32.load offset=4 (local.get $fun))))
      (br_if $failed (i32.eqz (local.get $head)))
      (local.set $head (call $resolve (local.get $head)))
      (br_if $failed (i32.ne (call $tag (local.get $head)) (global.get $T_INT)))
      (local.set $value (call $ival (local.get $head)))
      (br_if $failed (i32.ge_u (local.get $value) (i32.const 0x110000)))
      (call $bs_append_char (call $bval (i32.load (i32.const 0x3068))) (local.get $value))
      (local.set $node (call $resolve (i32.load (i32.const 0x3064))))
      (i32.store (i32.const 0x3064) (i32.load offset=4 (local.get $node)))
      (br $characters)))
  (i32.store (i32.const 0x3064) (i32.const 0))
  (i32.store (i32.const 0x3068) (i32.const 0))
  (local.get $result))

;; Recognize the displayException form of ExitFailure. This extends the
;; existing ExitSuccess message convention used by the C and Rust commands.
;; Return {status, recognized}. An explicit failure cannot report status zero.
(func $command_exit_code (param $address i32) (param $length i32) (result i32 i32)
  (local $index i32) (local $negative i32) (local $paren i32)
  (local $digits i32) (local $value i32) (local $byte i32)
  (if (i32.le_u (local.get $length) (i32.const 12))
    (then (return (i32.const 0) (i32.const 0))))
  (if (i32.eqz (call $names_equal (local.get $address) (global.get $command_exit_failure) (i32.const 12)))
    (then (return (i32.const 0) (i32.const 0))))
  (local.set $index (i32.const 12))
  (if (i32.eq (i32.load8_u (i32.add (local.get $address) (local.get $index))) (i32.const 40))
    (then (local.set $paren (i32.const 1)) (local.set $index (i32.add (local.get $index) (i32.const 1)))))
  (if (i32.lt_u (local.get $index) (local.get $length))
    (then
      (if (i32.eq (i32.load8_u (i32.add (local.get $address) (local.get $index))) (i32.const 45))
        (then (local.set $negative (i32.const 1)) (local.set $index (i32.add (local.get $index) (i32.const 1)))))))
  (block $number_done
    (loop $number
      (br_if $number_done (i32.eq (local.get $index) (local.get $length)))
      (local.set $byte (i32.load8_u (i32.add (local.get $address) (local.get $index))))
      (br_if $number_done (i32.gt_u (i32.sub (local.get $byte) (i32.const 48)) (i32.const 9)))
      (local.set $value (i32.add (i32.mul (local.get $value) (i32.const 10))
        (i32.sub (local.get $byte) (i32.const 48))))
      (local.set $digits (i32.const 1))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $number)))
  (if (local.get $paren)
    (then
      (if (i32.or (i32.eq (local.get $index) (local.get $length))
            (i32.ne (i32.load8_u (i32.add (local.get $address) (local.get $index))) (i32.const 41)))
        (then (return (i32.const 0) (i32.const 0))))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))))
  (if (i32.or (i32.eqz (local.get $digits)) (i32.ne (local.get $index) (local.get $length)))
    (then (return (i32.const 0) (i32.const 0))))
  (if (local.get $negative) (then (local.set $value (i32.sub (i32.const 0) (local.get $value)))))
  (select (local.get $value) (i32.const 1) (i32.ne (local.get $value) (i32.const 0)))
  (i32.const 1))

;; SomeException stores an existential value and its Exception dictionary.
;; Control.Exception.Internal's displaySomeException compiles to U(U(K2 A)).
;; Apply that graph, then collect its String through separate outer evaluations.
;; Keep the original exception at 0x3060 because each $execute clears 0x3004.
(func $command_exception (param $exception i32)
  (local $message i32) (local $descriptor i32) (local $address i32)
  (local $length i32) (local $code i32) (local $recognized i32)
  (local.set $exception (call $resolve (local.get $exception)))
  (i32.store (i32.const 0x3060) (local.get $exception))
  (if (i32.eq (call $tag (local.get $exception)) (global.get $T_INT))
    (then
      (local.set $code (call $ival (local.get $exception)))
      (local.set $address
        (if (result i32) (i32.lt_u (local.get $code) (i32.const 10))
          (then (i32.load (i32.add (i32.const 0xc200) (i32.shl (local.get $code) (i32.const 2)))))
          (else (global.get $command_unknown))))
      (local.set $message (call $bytes_view (local.get $address) (call $strlen (local.get $address)))))
    (else
      (local.set $message (call $collect_string
        (call $ap
          (call $ap (call $prim (global.get $T_U))
            (call $ap (call $prim (global.get $T_U))
              (call $ap (call $prim (global.get $T_K2)) (call $prim (global.get $T_A)))))
          (local.get $exception))))))
  (if (i32.eqz (local.get $message))
    (then
      (local.set $message (call $bytes_view (global.get $command_display_failed)
        (call $strlen (global.get $command_display_failed))))))
  (i32.store (i32.const 0x3070) (local.get $message))
  (i32.store (i32.const 0x3004) (i32.load (i32.const 0x3060)))
  (local.set $descriptor (call $bval (local.get $message)))
  (local.set $address (i32.load offset=4 (local.get $descriptor)))
  (local.set $length (i32.load (local.get $descriptor)))
  (call $wasi_flush)
  (if (i32.and (i32.eq (local.get $length) (i32.const 11))
        (call $names_equal (local.get $address) (global.get $command_exit_success) (i32.const 11)))
    (then (call $wasi_proc_exit (i32.const 0)) (unreachable)))
  (call $command_exit_code (local.get $address) (local.get $length))
  (local.set $recognized)
  (local.set $code)
  (if (local.get $recognized)
    (then (call $wasi_proc_exit (local.get $code)) (unreachable)))
  ;; The collected message is already UTF-8. Write bytes directly to stderr.
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc125) (i32.const 1)))
  (drop (call $wasi_write_all (i32.const 2) (global.get $wasi_progname)
    (call $strlen (global.get $wasi_progname))))
  (drop (call $wasi_write_all (i32.const 2) (global.get $command_uncaught_prefix)
    (call $strlen (global.get $command_uncaught_prefix))))
  (drop (call $wasi_write_all (i32.const 2) (local.get $address) (local.get $length)))
  (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc125) (i32.const 1)))
  (call $wasi_proc_exit (i32.const 1))
  (unreachable))

(func $start (export "_start")
  (local $input i32) (local $length i32) (local $path i32)
  (local $skip i32) (local $root i32) (local $result i32)
  (call $initialize)
  (if (i32.lt_u (global.get $wasi_argc) (i32.const 4))
    (then
      (drop (call $wasi_write_all (i32.const 2) (i32.const 0xc160)
        (call $strlen (i32.const 0xc160))))
      (call $wasi_proc_exit (i32.const 2))
      (unreachable)))
  (local.set $path (i32.load offset=4 (global.get $wasi_argv)))
  (local.set $skip (i32.const 2))
  (local.set $input (i32.load offset=8 (global.get $wasi_argv)))
  (if (i32.and
        (i32.eq (i32.load16_u align=1 (local.get $input)) (i32.const 0x2d2d))
        (i32.eqz (i32.load8_u offset=2 (local.get $input))))
    (then (local.set $skip (i32.const 3)))
    (else (call $fail (i32.const 35))))
  (call $wasi_read_file (local.get $path) (call $strlen (local.get $path)))
  (local.set $length)
  (local.set $input)
  (if (i32.eqz (local.get $input)) (then (call $fail (i32.const 36))))
  (local.set $root (call $parse (local.get $input) (local.get $length)))
  (i32.store (i32.const 0x154) (local.get $root))
  (call $blob_free (local.get $input))
  (i32.store (i32.const 0x15c) (call $wasi_args_setup (local.get $skip)))
  (local.set $result (call $execute
    (call $ap (local.get $root) (i32.load (i32.const 0x3000)))))
  (call $wasi_flush)
  (if (i32.eqz (local.get $result))
    (then (call $command_exception (i32.load (i32.const 0x3004))))))

(export "parse" (func $parse))
(export "allocate" (func $blob_alloc))
(export "collect" (func $gc))
