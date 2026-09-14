;; Load the uncompressed v8.4 postfix format.
;; Parsing uses the free part of the application stack. It cannot collect.
;; Temporary labels are payload allocations. They are released on success.
;; All input reads, stack operations, and label probes have bounds checks.
;; Failure codes: 100 truncated input; 101 version; 102 syntax; 103 number;
;; 104 stack; 105 labels; 106 unknown name; 107 unresolved label;
;; 108 unsupported Integer literal; 109 invalid input range.
(global $parse_status (mut i32) (i32.const 0))
(global $parse_cursor (mut i32) (i32.const 0))
(global $parse_end (mut i32) (i32.const 0))
(global $parse_sp (mut i32) (i32.const -1))
(global $parse_base (mut i32) (i32.const -1))
(global $parse_labels (mut i32) (i32.const 0))
(global $parse_label_slots (mut i32) (i32.const 0))
(global $parse_label_limit (mut i32) (i32.const 0))
(global $parse_label_count (mut i32) (i32.const 0))

;; Retain the first syntax error. Helpers return a bounded fallback, and the
;; record parser exits through one cleanup block. No host exception is needed.
(func $parse_reject (param $code i32)
  (if (i32.eqz (global.get $parse_status))
    (then (global.set $parse_status (local.get $code)))))

;; Read the next byte without advancing. Return -1 at the input boundary.
(func $parse_peek (result i32)
  (if (global.get $parse_status) (then (return (i32.const -1))))
  (if (result i32) (i32.lt_u (global.get $parse_cursor) (global.get $parse_end))
    (then (i32.load8_u (global.get $parse_cursor)))
    (else (i32.const -1))))

;; Read one byte. A missing byte is a parse failure.
(func $parse_take (result i32)
  (local $byte i32)
  (local.set $byte (call $parse_peek))
  (if (i32.eq (local.get $byte) (i32.const -1))
    (then (call $parse_reject (i32.const 100)) (return (i32.const -1))))
  (global.set $parse_cursor (i32.add (global.get $parse_cursor) (i32.const 1)))
  (local.get $byte))

;; Treat CRLF and ordinary ASCII spacing consistently at token boundaries.
(func $parse_space (param $byte i32) (result i32)
  (i32.or (i32.eq (local.get $byte) (i32.const 32))
    (i32.or (i32.eq (local.get $byte) (i32.const 9))
      (i32.or (i32.eq (local.get $byte) (i32.const 10))
        (i32.eq (local.get $byte) (i32.const 13))))))

;; Header lines require LF. Accept either adjacent CR convention.
(func $parse_newline
  (if (i32.eq (call $parse_peek) (i32.const 13))
    (then (drop (call $parse_take))))
  (if (i32.ne (call $parse_take) (i32.const 10))
    (then (call $parse_reject (i32.const 102)) (return)))
  (if (i32.eq (call $parse_peek) (i32.const 13))
    (then (drop (call $parse_take)))))

;; Signed scalar literals wrap at 64 bits, as the serialized integer format does.
;; Sizes and labels must instead fit in an unsigned 32-bit word.
(func $parse_number (param $bounded i32) (result i64)
  (local $value i64) (local $negative i32) (local $digits i32) (local $byte i32)
  (if (i32.eq (call $parse_peek) (i32.const 45))
    (then
      (if (local.get $bounded)
        (then (call $parse_reject (i32.const 103)) (return (i64.const 0))))
      (local.set $negative (i32.const 1))
      (drop (call $parse_take))))
  (block $done
    (loop $digits_loop
      (local.set $byte (call $parse_peek))
      (br_if $done (i32.gt_u (i32.sub (local.get $byte) (i32.const 48)) (i32.const 9)))
      (local.set $byte (i32.sub (local.get $byte) (i32.const 48)))
      (if (local.get $bounded)
        (then
          (if (i32.or (i64.gt_u (local.get $value) (i64.const 429496729))
                (i32.and (i64.eq (local.get $value) (i64.const 429496729))
                  (i32.gt_u (local.get $byte) (i32.const 5))))
            (then (call $parse_reject (i32.const 103)) (return (i64.const 0))))))
      (local.set $value (i64.add (i64.mul (local.get $value) (i64.const 10))
        (i64.extend_i32_u (local.get $byte))))
      (local.set $digits (i32.const 1))
      (drop (call $parse_take))
      (br $digits_loop)))
  (if (i32.eqz (local.get $digits))
    (then (call $parse_reject (i32.const 103)) (return (i64.const 0))))
  (if (result i64) (local.get $negative)
    (then (i64.sub (i64.const 0) (local.get $value)))
    (else (local.get $value))))

;; Return the address and length of the next nonempty token.
(func $parse_word (result i32 i32)
  (local $start i32) (local $byte i32)
  (local.set $start (global.get $parse_cursor))
  (block $done
    (loop $bytes
      (local.set $byte (call $parse_peek))
      (br_if $done (i32.eq (local.get $byte) (i32.const -1)))
      (br_if $done (call $parse_space (local.get $byte)))
      (global.set $parse_cursor (i32.add (global.get $parse_cursor) (i32.const 1)))
      (br $bytes)))
  (if (i32.eq (local.get $start) (global.get $parse_cursor))
    (then (call $parse_reject (i32.const 102)) (return (i32.const 0) (i32.const 0))))
  (local.get $start)
  (i32.sub (global.get $parse_cursor) (local.get $start)))

;; Push above the caller's live application spine.
(func $parse_push (param $node i32)
  (if (global.get $parse_status) (then (return)))
  (global.set $parse_sp (i32.add (global.get $parse_sp) (i32.const 1)))
  (if (i32.ge_u (global.get $parse_sp) (i32.load (i32.const 0x108)))
    (then (call $parse_reject (i32.const 104)) (return)))
  (i32.store (i32.add (i32.const 0x10000)
    (i32.shl (global.get $parse_sp) (i32.const 2))) (local.get $node)))

;; Pop only values that this parse operation pushed.
(func $parse_pop (result i32)
  (local $node i32)
  (if (i32.le_s (global.get $parse_sp) (global.get $parse_base))
    (then (call $parse_reject (i32.const 104)) (return (i32.const 0))))
  (local.set $node (i32.load (i32.add (i32.const 0x10000)
    (i32.shl (global.get $parse_sp) (i32.const 2)))))
  (global.set $parse_sp (i32.sub (global.get $parse_sp) (i32.const 1)))
  (local.get $node))

;; Each label entry contains its numeric label, node, and definition flag.
;; The declared label count limits insertions and makes full-table failure finite.
(func $parse_find_label (param $label i32) (result i32)
  (local $slot i32) (local $entry i32) (local $probes i32)
  (local.set $slot (i32.rem_u (local.get $label) (global.get $parse_label_slots)))
  (loop $probe
    (if (i32.eq (local.get $probes) (global.get $parse_label_slots))
      (then (call $parse_reject (i32.const 105)) (return (i32.const 0))))
    (local.set $entry (i32.add (global.get $parse_labels)
      (i32.mul (local.get $slot) (i32.const 12))))
    (if (i32.eqz (i32.load offset=4 (local.get $entry)))
      (then
        (if (i32.ge_u (global.get $parse_label_count) (global.get $parse_label_limit))
          (then (call $parse_reject (i32.const 105)) (return (i32.const 0))))
        (global.set $parse_label_count (i32.add (global.get $parse_label_count) (i32.const 1)))
        (i32.store (local.get $entry) (local.get $label))
        (return (local.get $entry))))
    (if (i32.eq (i32.load (local.get $entry)) (local.get $label))
      (then (return (local.get $entry))))
    (local.set $slot (i32.add (local.get $slot) (i32.const 1)))
    (if (i32.eq (local.get $slot) (global.get $parse_label_slots))
      (then (local.set $slot (i32.const 0))))
    (local.set $probes (i32.add (local.get $probes) (i32.const 1)))
    (br $probe))
  (unreachable))

;; Count first, then allocate the exact decoded string size plus its trailing NUL.
;; Backslash quotes a byte; \? and \_ encode DEL and FF. Caret and bar encode
;; control and high bytes. These are the current ExpPrint.hs escape rules.
(func $parse_string (result i32)
  (local $cursor i32) (local $byte i32) (local $length i32)
  (local $buffer i32) (local $index i32)
  (local.set $cursor (global.get $parse_cursor))
  (block $counted
    (loop $count
      (if (i32.ge_u (local.get $cursor) (global.get $parse_end))
        (then (call $parse_reject (i32.const 100)) (return (i32.const 0))))
      (local.set $byte (i32.load8_u (local.get $cursor)))
      (local.set $cursor (i32.add (local.get $cursor) (i32.const 1)))
      (br_if $counted (i32.eq (local.get $byte) (i32.const 34)))
      (if (i32.or (i32.eq (local.get $byte) (i32.const 92))
            (i32.or (i32.eq (local.get $byte) (i32.const 94))
              (i32.eq (local.get $byte) (i32.const 124))))
        (then
          (if (i32.ge_u (local.get $cursor) (global.get $parse_end))
            (then (call $parse_reject (i32.const 100)) (return (i32.const 0))))
          (local.set $cursor (i32.add (local.get $cursor) (i32.const 1)))))
      (local.set $length (i32.add (local.get $length) (i32.const 1)))
      (br $count)))
  (local.set $buffer (call $blob_alloc (i32.add (local.get $length) (i32.const 1)) (i32.const 0)))
  (block $decoded
    (loop $decode
      (local.set $byte (call $parse_take))
      (br_if $decoded (i32.eq (local.get $byte) (i32.const 34)))
      (if (i32.eq (local.get $byte) (i32.const 92))
        (then
          (local.set $byte (call $parse_take))
          (if (i32.eq (local.get $byte) (i32.const 63))
            (then (local.set $byte (i32.const 127)))
            (else
              (if (i32.eq (local.get $byte) (i32.const 95))
                (then (local.set $byte (i32.const 255)))))))
        (else
          (if (i32.eq (local.get $byte) (i32.const 94))
            (then
              (local.set $byte (call $parse_take))
              (local.set $byte (i32.or (i32.and (local.get $byte) (i32.const 31))
                (if (result i32) (i32.lt_u (local.get $byte) (i32.const 64))
                  (then (i32.const 0)) (else (i32.const 128))))))
            (else
              (if (i32.eq (local.get $byte) (i32.const 124))
                (then (local.set $byte (i32.or (call $parse_take) (i32.const 128)))))))))
      (i32.store8 (i32.add (local.get $buffer) (local.get $index)) (local.get $byte))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $decode)))
  (i32.store8 (i32.add (local.get $buffer) (local.get $length)) (i32.const 0))
  (call $bytes_view (local.get $buffer) (local.get $length)))

;; Check the floating literal grammar before the exact numeric converter.
;; This keeps malformed stream input on the catchable Deserialize error path.
(func $parse_float_valid (param $address i32) (param $length i32) (result i32)
  (local $index i32) (local $byte i32) (local $digits i32) (local $point i32)
  (if (i32.eqz (local.get $length)) (then (return (i32.const 0))))
  (local.set $byte (i32.load8_u (local.get $address)))
  (if (i32.or (i32.eq (local.get $byte) (i32.const 43))
        (i32.eq (local.get $byte) (i32.const 45)))
    (then
      (local.set $address (i32.add (local.get $address) (i32.const 1)))
      (local.set $length (i32.sub (local.get $length) (i32.const 1)))))
  (if (i32.eq (local.get $length) (i32.const 3))
    (then
      (if (i32.or
            (i32.and
              (i32.and (i32.eq (call $fp_lower (local.get $address) (i32.const 0)) (i32.const 105))
                (i32.eq (call $fp_lower (local.get $address) (i32.const 1)) (i32.const 110)))
              (i32.eq (call $fp_lower (local.get $address) (i32.const 2)) (i32.const 102)))
            (i32.and
              (i32.and (i32.eq (call $fp_lower (local.get $address) (i32.const 0)) (i32.const 110))
                (i32.eq (call $fp_lower (local.get $address) (i32.const 1)) (i32.const 97)))
              (i32.eq (call $fp_lower (local.get $address) (i32.const 2)) (i32.const 110))))
        (then (return (i32.const 1))))))
  (if (i32.eq (local.get $length) (i32.const 8))
    (then
      (if (i32.and
            (i32.eq (i32.or (i32.load align=1 (local.get $address)) (i32.const 0x20202020))
              (i32.const 0x69666e69))
            (i32.eq (i32.or (i32.load offset=4 align=1 (local.get $address)) (i32.const 0x20202020))
              (i32.const 0x7974696e)))
        (then (return (i32.const 1))))))
  (block $mantissa_done
    (loop $mantissa
      (br_if $mantissa_done (i32.eq (local.get $index) (local.get $length)))
      (local.set $byte (i32.load8_u (i32.add (local.get $address) (local.get $index))))
      (if (i32.eq (local.get $byte) (i32.const 46))
        (then
          (if (local.get $point) (then (return (i32.const 0))))
          (local.set $point (i32.const 1))
          (local.set $index (i32.add (local.get $index) (i32.const 1)))
          (br $mantissa)))
      (br_if $mantissa_done (i32.gt_u (i32.sub (local.get $byte) (i32.const 48)) (i32.const 9)))
      (local.set $digits (i32.const 1))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $mantissa)))
  (if (i32.eqz (local.get $digits)) (then (return (i32.const 0))))
  (if (i32.eq (local.get $index) (local.get $length)) (then (return (i32.const 1))))
  (if (i32.ne (i32.or (local.get $byte) (i32.const 32)) (i32.const 101))
    (then (return (i32.const 0))))
  (local.set $index (i32.add (local.get $index) (i32.const 1)))
  (if (i32.lt_u (local.get $index) (local.get $length))
    (then
      (local.set $byte (i32.load8_u (i32.add (local.get $address) (local.get $index))))
      (if (i32.or (i32.eq (local.get $byte) (i32.const 43))
            (i32.eq (local.get $byte) (i32.const 45)))
        (then (local.set $index (i32.add (local.get $index) (i32.const 1)))))))
  (if (i32.eq (local.get $index) (local.get $length)) (then (return (i32.const 0))))
  (loop $exponent
    (if (i32.gt_u
          (i32.sub (i32.load8_u (i32.add (local.get $address) (local.get $index))) (i32.const 48))
          (i32.const 9))
      (then (return (i32.const 0))))
    (local.set $index (i32.add (local.get $index) (i32.const 1)))
    (br_if $exponent (i32.lt_u (local.get $index) (local.get $length))))
  (i32.const 1))

;; Resolve one program. Strict mode also checks all trailing input bytes.
;; The caller's stack pointer does not change. Prefix mode leaves the cursor
;; immediately after the closing brace, before any stream trailer.
(func $parse_record (param $address i32) (param $length i32) (param $complete i32) (result i32)
  (local $byte i32) (local $x i32) (local $y i32) (local $node i32)
  (local $start i32) (local $size i32) (local $entry i32) (local $index i32)
  (local $is_wide i32) (local $table_size i32) (local $root i32)
  (global.set $parse_status (i32.const 0))
  (global.set $parse_labels (i32.const 0))
  (block $parse_failed
  (if (i32.or
        (i64.gt_u (i64.add (i64.extend_i32_u (local.get $address))
                    (i64.extend_i32_u (local.get $length)))
          (i64.shl (i64.extend_i32_u (memory.size)) (i64.const 16)))
        (i32.lt_u (i32.add (local.get $address) (local.get $length)) (local.get $address)))
    (then (call $parse_reject (i32.const 109)) (br $parse_failed)))
  (global.set $parse_cursor (local.get $address))
  (global.set $parse_end (i32.add (local.get $address) (local.get $length)))
  (global.set $parse_base (i32.load (i32.const 0x104)))
  (global.set $parse_sp (global.get $parse_base))
  (if (i32.or (i32.lt_s (global.get $parse_base) (i32.const -1))
        (i32.ge_s (global.get $parse_base) (i32.load (i32.const 0x108))))
    (then (call $parse_reject (i32.const 104)) (br $parse_failed)))
  (if (i32.ne (call $parse_take) (i32.const 118))
    (then (call $parse_reject (i32.const 101)) (br $parse_failed)))
  (if (i32.ne (call $parse_take) (i32.const 56))
    (then (call $parse_reject (i32.const 101)) (br $parse_failed)))
  (if (i32.ne (call $parse_take) (i32.const 46))
    (then (call $parse_reject (i32.const 101)) (br $parse_failed)))
  (if (i32.ne (call $parse_take) (i32.const 52))
    (then (call $parse_reject (i32.const 101)) (br $parse_failed)))
  (call $parse_newline)
  (global.set $parse_label_limit (i32.wrap_i64 (call $parse_number (i32.const 1))))
  (call $parse_newline)
  (if (i32.or (i32.gt_u (global.get $parse_label_limit) (local.get $length))
        (i32.gt_u (global.get $parse_label_limit) (i32.const 119304646)))
    (then (call $parse_reject (i32.const 105)) (br $parse_failed)))
  (global.set $parse_label_slots (i32.mul (global.get $parse_label_limit) (i32.const 3)))
  (if (i32.eqz (global.get $parse_label_slots))
    (then (global.set $parse_label_slots (i32.const 1))))
  (local.set $table_size (i32.mul (global.get $parse_label_slots) (i32.const 12)))
  (global.set $parse_labels (call $blob_alloc (local.get $table_size) (i32.const 0)))
  (memory.fill (global.get $parse_labels) (i32.const 0) (local.get $table_size))
  (global.set $parse_label_count (i32.const 0))
  (block $program_done
    (loop $tokens
      (br_if $parse_failed (global.get $parse_status))
      (local.set $byte (call $parse_take))
      (br_if $tokens (call $parse_space (local.get $byte)))
      (if (i32.eq (local.get $byte) (i32.const 64))
        (then
          (local.set $x (call $parse_pop))
          (local.set $y (call $parse_pop))
          (br_if $parse_failed (global.get $parse_status))
          (call $parse_push (call $ap (local.get $y) (local.get $x)))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 125))
        (then
          (local.set $root (call $parse_pop))
          (if (i32.ne (global.get $parse_sp) (global.get $parse_base))
            (then (call $parse_reject (i32.const 104)) (br $parse_failed)))
          (br $program_done)))
      (if (i32.eq (local.get $byte) (i32.const 35))
        (then
          (local.set $is_wide (i32.eq (call $parse_peek) (i32.const 35)))
          (if (local.get $is_wide) (then (drop (call $parse_take))))
          (call $parse_push
            (if (result i32) (local.get $is_wide)
              (then (call $int64 (call $parse_number (i32.const 0))))
              (else (call $int (i32.wrap_i64 (call $parse_number (i32.const 0)))))))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 38))
        (then
          (local.set $is_wide (i32.eq (call $parse_peek) (i32.const 38)))
          (if (local.get $is_wide) (then (drop (call $parse_take))))
          (call $parse_word)
          (local.set $size)
          (local.set $start)
          (br_if $parse_failed (global.get $parse_status))
          (if (i32.eqz (call $parse_float_valid (local.get $start) (local.get $size)))
            (then (call $parse_reject (i32.const 103)) (br $parse_failed)))
          (call $parse_push
            (if (result i32) (local.get $is_wide)
              (then (call $float (call $parse_f32 (local.get $start) (local.get $size))))
              (else (call $double (call $parse_f64 (local.get $start) (local.get $size))))))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 91))
        (then
          (local.set $size (i32.wrap_i64 (call $parse_number (i32.const 1))))
          (if (i32.ne (call $parse_take) (i32.const 93))
            (then (call $parse_reject (i32.const 102)) (br $parse_failed)))
          (if (i32.gt_u (local.get $size)
                (i32.sub (global.get $parse_sp) (global.get $parse_base)))
            (then (call $parse_reject (i32.const 104)) (br $parse_failed)))
          (local.set $entry (call $blob_alloc
            (i32.shl (i32.add (local.get $size) (i32.const 1)) (i32.const 2)) (i32.const -1)))
          (i32.store (local.get $entry) (local.get $size))
          (local.set $index (local.get $size))
          (block $array_done
            (loop $array
              (br_if $array_done (i32.eqz (local.get $index)))
              (i32.store (i32.add (local.get $entry) (i32.shl (local.get $index) (i32.const 2)))
                (call $parse_pop))
              (local.set $index (i32.sub (local.get $index) (i32.const 1)))
              (br $array)))
          (local.set $node (call $node (global.get $T_ARR)))
          (i32.store offset=4 (local.get $node) (local.get $entry))
          (call $parse_push (local.get $node))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 95))
        (then
          (local.set $entry (call $parse_find_label (i32.wrap_i64 (call $parse_number (i32.const 1)))))
          (br_if $parse_failed (global.get $parse_status))
          (local.set $node (i32.load offset=4 (local.get $entry)))
          (if (i32.eqz (local.get $node))
            (then
              (local.set $node (call $node (global.get $T_FREE)))
              (i32.store (local.get $node) (i32.const 2))
              (i32.store offset=4 (local.get $entry) (local.get $node))))
          (call $parse_push (local.get $node))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 58))
        (then
          (local.set $entry (call $parse_find_label (i32.wrap_i64 (call $parse_number (i32.const 1)))))
          (br_if $parse_failed (global.get $parse_status))
          (if (i32.ne (call $parse_take) (i32.const 32))
            (then (call $parse_reject (i32.const 102)) (br $parse_failed)))
          (if (i32.load offset=8 (local.get $entry))
            (then (call $parse_reject (i32.const 105)) (br $parse_failed)))
          (local.set $x (call $parse_pop))
          (call $parse_push (local.get $x))
          (local.set $node (i32.load offset=4 (local.get $entry)))
          (if (local.get $node)
            (then (i32.store (local.get $node) (i32.or (local.get $x) (i32.const 2))))
            (else (i32.store offset=4 (local.get $entry) (local.get $x))))
          (i32.store offset=8 (local.get $entry) (i32.const 1))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 34))
        (then (call $parse_push (call $parse_string)) (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 36))
        (then
          (local.set $size (i32.wrap_i64 (call $parse_number (i32.const 1))))
          (if (i32.ne (call $parse_take) (i32.const 32))
            (then (call $parse_reject (i32.const 102)) (br $parse_failed)))
          (if (i32.gt_u (local.get $size)
                (i32.sub (global.get $parse_end) (global.get $parse_cursor)))
            (then (call $parse_reject (i32.const 100)) (br $parse_failed)))
          (call $parse_push (call $bytes (global.get $parse_cursor) (local.get $size)))
          (global.set $parse_cursor (i32.add (global.get $parse_cursor) (local.get $size)))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 33))
        (then
          (if (i32.ne (call $parse_take) (i32.const 34))
            (then (call $parse_reject (i32.const 102)) (br $parse_failed)))
          (local.set $x (call $parse_string))
          (local.set $node (call $node (global.get $T_TICK)))
          (i32.store offset=4 (local.get $node) (local.get $x))
          (call $parse_push (local.get $node))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 94))
        (then
          (call $parse_word)
          (local.set $size)
          (local.set $start)
          (local.set $x (call $service_lookup (local.get $start) (local.get $size)))
          (if (i32.eq (local.get $x) (i32.const -1))
            (then (call $parse_reject (i32.const 106)) (br $parse_failed)))
          (local.set $node (call $node (global.get $T_IO_CCALL)))
          (i32.store offset=4 (local.get $node) (local.get $x))
          (call $parse_push (local.get $node))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 59))
        (then
          (call $parse_word)
          (local.set $size)
          (local.set $start)
          (local.set $x (i32.const 0))
          (if (i32.eqz (i32.and (i32.eq (local.get $size) (i32.const 1))
                (i32.eq (i32.load8_u (local.get $start)) (i32.const 48))))
            (then
              (if (i32.ne (call $service_lookup (local.get $start) (local.get $size))
                    (global.get $svc_closeb))
                (then (call $parse_reject (i32.const 106)) (br $parse_failed)))
              (local.set $x (i32.add (global.get $svc_closeb) (i32.const 1)))))
          (local.set $node (call $node (global.get $T_FUNPTR)))
          (i32.store offset=4 (local.get $node) (local.get $x))
          (call $parse_push (local.get $node))
          (br $tokens)))
      (if (i32.eq (local.get $byte) (i32.const 37))
        (then (call $parse_reject (i32.const 108)) (br $parse_failed)))
      (global.set $parse_cursor (i32.sub (global.get $parse_cursor) (i32.const 1)))
      (call $parse_word)
      (local.set $size)
      (local.set $start)
      (local.set $node (call $prim_lookup (local.get $start) (local.get $size)))
      (if (i32.eqz (local.get $node))
        (then (call $parse_reject (i32.const 106)) (br $parse_failed)))
      (call $parse_push (local.get $node))
      (br $tokens)))
  (local.set $index (i32.const 0))
  (block $labels_done
    (loop $labels
      (br_if $labels_done (i32.eq (local.get $index) (global.get $parse_label_slots)))
      (local.set $entry (i32.add (global.get $parse_labels) (i32.mul (local.get $index) (i32.const 12))))
      (if (i32.and (i32.ne (i32.load offset=4 (local.get $entry)) (i32.const 0))
            (i32.eqz (i32.load offset=8 (local.get $entry))))
        (then (call $parse_reject (i32.const 107)) (br $parse_failed)))
      (local.set $index (i32.add (local.get $index) (i32.const 1)))
      (br $labels)))
  (if (local.get $complete)
    (then
      (block $input_done
        (loop $trailing
          (br_if $input_done (i32.eq (global.get $parse_cursor) (global.get $parse_end)))
          (if (i32.eqz (call $parse_space (call $parse_take)))
            (then (call $parse_reject (i32.const 102)) (br $parse_failed)))
          (br $trailing)))))
  (call $blob_free (global.get $parse_labels))
  (global.set $parse_labels (i32.const 0))
  (return (local.get $root)))
  (if (global.get $parse_labels)
    (then (call $blob_free (global.get $parse_labels))))
  (global.set $parse_labels (i32.const 0))
  (i32.const 0))

;; Load one complete byte buffer. Non-whitespace trailers are rejected.
(func $parse (param $address i32) (param $length i32) (result i32)
  (local $root i32)
  (local.set $root (call $parse_record (local.get $address) (local.get $length) (i32.const 1)))
  (if (i32.eqz (local.get $root))
    (then (call $fail (global.get $parse_status)) (unreachable)))
  (local.get $root))

;; Load one record from a byte buffer. The caller can retain the unread suffix
;; at $parse_cursor. Raw and quoted byte strings can contain closing braces.
(func $parse_prefix (param $address i32) (param $length i32) (result i32)
  (local $root i32)
  (local.set $root (call $parse_record (local.get $address) (local.get $length) (i32.const 0)))
  (if (result i32) (local.get $root)
    (then (local.get $root))
    (else (call $raise_rts (i32.const 9)))))
