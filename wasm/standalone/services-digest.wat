;; MD5 implements the existing content-digest services used by the compiler.
;; The four-word state, 64 round constants, rotation counts, and padding follow
;; RFC1321. This digest is used for content identity, not password storage.
;; Source reference: https://www.rfc-editor.org/rfc/rfc1321#section-3
;; Context layout: state[4] at0, input bit count at16, buffered length at24,
;; and a64-byte partial block at32. All arithmetic wraps modulo2^32.
;; No function evaluates graph nodes or invokes collection.

(func $md5_init (result i32)
  (local $ctx i32)
  (local.set $ctx (call $blob_alloc (i32.const 96) (i32.const 0)))
  (i32.store (local.get $ctx) (i32.const 0x67452301))
  (i32.store offset=4 (local.get $ctx) (i32.const 0xefcdab89))
  (i32.store offset=8 (local.get $ctx) (i32.const 0x98badcfe))
  (i32.store offset=12 (local.get $ctx) (i32.const 0x10325476))
  (local.get $ctx))

;; Compress one complete block. INPUT can refer directly to an input buffer.
(func $md5_block (param $ctx i32) (param $input i32)
  (local $a i32) (local $b i32) (local $c i32) (local $d i32)
  (local $f i32) (local $g i32) (local $i i32) (local $old_d i32)
  (local.set $a (i32.load (local.get $ctx)))
  (local.set $b (i32.load offset=4 (local.get $ctx)))
  (local.set $c (i32.load offset=8 (local.get $ctx)))
  (local.set $d (i32.load offset=12 (local.get $ctx)))
  (loop $round
    (if (i32.lt_u (local.get $i) (i32.const 16))
      (then
        (local.set $f (i32.or (i32.and (local.get $b) (local.get $c))
          (i32.and (i32.xor (local.get $b) (i32.const -1)) (local.get $d))))
        (local.set $g (local.get $i)))
      (else
        (if (i32.lt_u (local.get $i) (i32.const 32))
          (then
            (local.set $f (i32.or (i32.and (local.get $b) (local.get $d))
              (i32.and (local.get $c) (i32.xor (local.get $d) (i32.const -1)))))
            (local.set $g (i32.add (i32.mul (local.get $i) (i32.const 5)) (i32.const 1))))
          (else
            (if (i32.lt_u (local.get $i) (i32.const 48))
              (then
                (local.set $f (i32.xor (i32.xor (local.get $b) (local.get $c)) (local.get $d)))
                (local.set $g (i32.add (i32.mul (local.get $i) (i32.const 3)) (i32.const 5))))
              (else
                (local.set $f (i32.xor (local.get $c)
                  (i32.or (local.get $b) (i32.xor (local.get $d) (i32.const -1)))))
                (local.set $g (i32.mul (local.get $i) (i32.const 7)))))))))
    (local.set $g (i32.and (local.get $g) (i32.const 15)))
    (local.set $old_d (local.get $d))
    (local.set $d (local.get $c))
    (local.set $c (local.get $b))
    (local.set $b
      (i32.add (local.get $b)
        (i32.rotl
          (i32.add (i32.add (local.get $a) (local.get $f))
            (i32.add
              (i32.load (i32.add (i32.const 0xe000) (i32.shl (local.get $i) (i32.const 2))))
              (i32.load align=1 (i32.add (local.get $input) (i32.shl (local.get $g) (i32.const 2))))))
          (i32.load (i32.add (i32.const 0xe100) (i32.shl (local.get $i) (i32.const 2)))))))
    (local.set $a (local.get $old_d))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br_if $round (i32.lt_u (local.get $i) (i32.const 64))))
  (i32.store (local.get $ctx) (i32.add (i32.load (local.get $ctx)) (local.get $a)))
  (i32.store offset=4 (local.get $ctx) (i32.add (i32.load offset=4 (local.get $ctx)) (local.get $b)))
  (i32.store offset=8 (local.get $ctx) (i32.add (i32.load offset=8 (local.get $ctx)) (local.get $c)))
  (i32.store offset=12 (local.get $ctx) (i32.add (i32.load offset=12 (local.get $ctx)) (local.get $d))))

(func $md5_update (param $ctx i32) (param $input i32) (param $length i32)
  (local $used i32) (local $take i32)
  (i64.store offset=16 (local.get $ctx)
    (i64.add (i64.load offset=16 (local.get $ctx))
      (i64.shl (i64.extend_i32_u (local.get $length)) (i64.const 3))))
  (local.set $used (i32.load offset=24 (local.get $ctx)))
  (block $done
    (loop $chunk
      (br_if $done (i32.eqz (local.get $length)))
      (if (i32.and (i32.eqz (local.get $used))
            (i32.ge_u (local.get $length) (i32.const 64)))
        (then
          (call $md5_block (local.get $ctx) (local.get $input))
          (local.set $take (i32.const 64)))
        (else
          (local.set $take (i32.sub (i32.const 64) (local.get $used)))
          (local.set $take (select (local.get $length) (local.get $take)
            (i32.lt_u (local.get $length) (local.get $take))))
          (memory.copy (i32.add (local.get $ctx) (i32.add (i32.const 32) (local.get $used)))
            (local.get $input) (local.get $take))
          (local.set $used (i32.add (local.get $used) (local.get $take)))
          (if (i32.eq (local.get $used) (i32.const 64))
            (then
              (call $md5_block (local.get $ctx) (i32.add (local.get $ctx) (i32.const 32)))
              (local.set $used (i32.const 0))))))
      (local.set $input (i32.add (local.get $input) (local.get $take)))
      (local.set $length (i32.sub (local.get $length) (local.get $take)))
      (br $chunk)))
  (i32.store offset=24 (local.get $ctx) (local.get $used)))

(func $md5_finish (param $ctx i32) (param $output i32)
  (local $used i32) (local $buffer i32)
  (local.set $used (i32.load offset=24 (local.get $ctx)))
  (local.set $buffer (i32.add (local.get $ctx) (i32.const 32)))
  (i32.store8 (i32.add (local.get $buffer) (local.get $used)) (i32.const 128))
  (local.set $used (i32.add (local.get $used) (i32.const 1)))
  (if (i32.gt_u (local.get $used) (i32.const 56))
    (then
      (memory.fill (i32.add (local.get $buffer) (local.get $used)) (i32.const 0)
        (i32.sub (i32.const 64) (local.get $used)))
      (call $md5_block (local.get $ctx) (local.get $buffer))
      (local.set $used (i32.const 0))))
  (memory.fill (i32.add (local.get $buffer) (local.get $used)) (i32.const 0)
    (i32.sub (i32.const 56) (local.get $used)))
  (i64.store offset=56 (local.get $buffer) (i64.load offset=16 (local.get $ctx)))
  (call $md5_block (local.get $ctx) (local.get $buffer))
  (memory.copy (local.get $output) (local.get $ctx) (i32.const 16)))

(func $service_digest (param $id i32) (param $args i32) (result i32)
  (local $ctx i32) (local $input i32) (local $output i32) (local $length i32)
  (local $handle i32) (local $byte i32)
  (if (i32.and
        (i32.ne (local.get $id) (global.get $svc_md5Array))
        (i32.and (i32.ne (local.get $id) (global.get $svc_md5String))
          (i32.ne (local.get $id) (global.get $svc_md5BFILE))))
    (then (return (i32.const 0))))
  ;; All three library bindings pass input or BFILE first, output second.
  (local.set $input (call $pval (call $value_arg (local.get $args) (i32.const 0))))
  (local.set $output (call $pval (call $value_arg (local.get $args) (i32.const 1))))
  (local.set $ctx (call $md5_init))
  (if (i32.eq (local.get $id) (global.get $svc_md5BFILE))
    (then
      (local.set $handle (local.get $input))
      (local.set $input (call $blob_alloc (i32.const 64) (i32.const 0)))
      (block $eof
        (loop $read
          (local.set $byte (call $bfile_get (local.get $handle)))
          (br_if $eof (i32.eq (local.get $byte) (i32.const -1)))
          (i32.store8 (i32.add (local.get $input) (local.get $length)) (local.get $byte))
          (local.set $length (i32.add (local.get $length) (i32.const 1)))
          (if (i32.eq (local.get $length) (i32.const 64))
            (then
              (call $md5_update (local.get $ctx) (local.get $input) (local.get $length))
              (local.set $length (i32.const 0))))
          (br $read)))
      (call $md5_update (local.get $ctx) (local.get $input) (local.get $length))
      (call $blob_free (local.get $input)))
    (else
      (if (i32.eq (local.get $id) (global.get $svc_md5Array))
        (then (local.set $length (call $ival (call $value_arg (local.get $args) (i32.const 2)))))
        (else (local.set $length (call $strlen (local.get $input)))))
      (call $md5_update (local.get $ctx) (local.get $input) (local.get $length))))
  (call $md5_finish (local.get $ctx) (local.get $output))
  (call $blob_free (local.get $ctx))
  (call $prim (global.get $T_I)))

(data (i32.const 0xe000) "\78\a4\6a\d7\56\b7\c7\e8\db\70\20\24\ee\ce\bd\c1\af\0f\7c\f5\2a\c6\87\47\13\46\30\a8\01\95\46\fd\d8\98\80\69\af\f7\44\8b\b1\5b\ff\ff\be\d7\5c\89\22\11\90\6b\93\71\98\fd\8e\43\79\a6\21\08\b4\49\62\25\1e\f6\40\b3\40\c0\51\5a\5e\26\aa\c7\b6\e9\5d\10\2f\d6\53\14\44\02\81\e6\a1\d8\c8\fb\d3\e7\e6\cd\e1\21\d6\07\37\c3\87\0d\d5\f4\ed\14\5a\45\05\e9\e3\a9\f8\a3\ef\fc\d9\02\6f\67\8a\4c\2a\8d\42\39\fa\ff\81\f6\71\87\22\61\9d\6d\0c\38\e5\fd\44\ea\be\a4\a9\cf\de\4b\60\4b\bb\f6\70\bc\bf\be\c6\7e\9b\28\fa\27\a1\ea\85\30\ef\d4\05\1d\88\04\39\d0\d4\d9\e5\99\db\e6\f8\7c\a2\1f\65\56\ac\c4\44\22\29\f4\97\ff\2a\43\a7\23\94\ab\39\a0\93\fc\c3\59\5b\65\92\cc\0c\8f\7d\f4\ef\ff\d1\5d\84\85\4f\7e\a8\6f\e0\e6\2c\fe\14\43\01\a3\a1\11\08\4e\82\7e\53\f7\35\f2\3a\bd\bb\d2\d7\2a\91\d3\86\eb")
(data (i32.const 0xe100) "\07\00\00\00\0c\00\00\00\11\00\00\00\16\00\00\00\07\00\00\00\0c\00\00\00\11\00\00\00\16\00\00\00\07\00\00\00\0c\00\00\00\11\00\00\00\16\00\00\00\07\00\00\00\0c\00\00\00\11\00\00\00\16\00\00\00\05\00\00\00\09\00\00\00\0e\00\00\00\14\00\00\00\05\00\00\00\09\00\00\00\0e\00\00\00\14\00\00\00\05\00\00\00\09\00\00\00\0e\00\00\00\14\00\00\00\05\00\00\00\09\00\00\00\0e\00\00\00\14\00\00\00\04\00\00\00\0b\00\00\00\10\00\00\00\17\00\00\00\04\00\00\00\0b\00\00\00\10\00\00\00\17\00\00\00\04\00\00\00\0b\00\00\00\10\00\00\00\17\00\00\00\04\00\00\00\0b\00\00\00\10\00\00\00\17\00\00\00\06\00\00\00\0a\00\00\00\0f\00\00\00\15\00\00\00\06\00\00\00\0a\00\00\00\0f\00\00\00\15\00\00\00\06\00\00\00\0a\00\00\00\0f\00\00\00\15\00\00\00\06\00\00\00\0a\00\00\00\0f\00\00\00\15\00\00\00")
