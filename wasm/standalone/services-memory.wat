;; Fixed memory and ABI-independent runtime services.
;; These names come from existing bytecode. Every case executes WAT operations;
;; there is no dynamic C symbol lookup and no host C function-pointer table.
;; Raw allocations scan aligned words conservatively because Haskell can store
;; pointers with pokePtr. Byte buffers created by bytestring operations do not.
;; Raw memory access has the normal unsafe Ptr contract and WASM bounds checks.
;; Return zero for services handled by another component.


(data (i32.const 0xe400) "\50\65\72\6d\69\73\73\69\6f\6e\20\64\65\6e\69\65\64\00\42\61\64\20\66\69\6c\65\20\64\65\73\63\72\69\70\74\6f\72\00\46\69\6c\65\20\65\78\69\73\74\73\00\49\6e\76\61\6c\69\64\20\61\72\67\75\6d\65\6e\74\00\49\73\20\61\20\64\69\72\65\63\74\6f\72\79\00\4e\6f\20\73\75\63\68\20\66\69\6c\65\20\6f\72\20\64\69\72\65\63\74\6f\72\79\00\4e\6f\74\20\65\6e\6f\75\67\68\20\73\70\61\63\65\00\46\75\6e\63\74\69\6f\6e\20\6e\6f\74\20\73\75\70\70\6f\72\74\65\64\00\4e\6f\74\20\61\20\64\69\72\65\63\74\6f\72\79\00\44\69\72\65\63\74\6f\72\79\20\6e\6f\74\20\65\6d\70\74\79\00\4f\70\65\72\61\74\69\6f\6e\20\6e\6f\74\20\73\75\70\70\6f\72\74\65\64\00\52\65\73\75\6c\74\20\74\6f\6f\20\6c\61\72\67\65\00\52\65\61\64\2d\6f\6e\6c\79\20\66\69\6c\65\20\73\79\73\74\65\6d\00\57\41\53\49\20\65\72\72\6f\72\00")

(data (i32.const 0xe700) "Operation not permitted\00Value too large for data type\00")

(func $errno_message (param $code i32) (result i32)
  (if (i32.eq (local.get $code) (i32.const 2)) (then (return (i32.const 58368))))
  (if (i32.eq (local.get $code) (i32.const 8)) (then (return (i32.const 58386))))
  (if (i32.eq (local.get $code) (i32.const 20)) (then (return (i32.const 58406))))
  (if (i32.eq (local.get $code) (i32.const 28)) (then (return (i32.const 58418))))
  (if (i32.eq (local.get $code) (i32.const 31)) (then (return (i32.const 58435))))
  (if (i32.eq (local.get $code) (i32.const 44)) (then (return (i32.const 58450))))
  (if (i32.eq (local.get $code) (i32.const 48)) (then (return (i32.const 58476))))
  (if (i32.eq (local.get $code) (i32.const 52)) (then (return (i32.const 58493))))
  (if (i32.eq (local.get $code) (i32.const 54)) (then (return (i32.const 58516))))
  (if (i32.eq (local.get $code) (i32.const 55)) (then (return (i32.const 58532))))
  (if (i32.eq (local.get $code) (i32.const 58)) (then (return (i32.const 58552))))
  (if (i32.eq (local.get $code) (i32.const 61)) (then (return (i32.const 0xe718))))
  (if (i32.eq (local.get $code) (i32.const 63)) (then (return (i32.const 0xe700))))
  (if (i32.eq (local.get $code) (i32.const 68)) (then (return (i32.const 58576))))
  (if (i32.eq (local.get $code) (i32.const 69)) (then (return (i32.const 58593))))
  (i32.const 58615))


(func $service_memory (param $id i32) (param $args i32) (result i32)
  (local $p i32) (local $q i32) (local $len i32) (local $oldlen i32) (local $block i32) (local $wide i64)
  (block $unknown
    (block $s260
    (block $s259
    (block $s258
    (block $s257
    (block $s256
    (block $s255
    (block $s254
    (block $s253
    (block $s252
    (block $s251
    (block $s250
    (block $s249
    (block $s248
    (block $s247
    (block $s246
    (block $s245
    (block $s244
    (block $s243
    (block $s242
    (block $s241
    (block $s240
    (block $s239
    (block $s238
    (block $s237
    (block $s236
    (block $s235
    (block $s234
    (block $s233
    (block $s232
    (block $s231
    (block $s230
    (block $s229
    (block $s228
    (block $s227
    (block $s226
    (block $s225
    (block $s224
    (block $s223
    (block $s222
    (block $s221
    (block $s220
    (block $s219
    (block $s218
    (block $s217
    (block $s216
    (block $s215
    (block $s214
    (block $s213
    (block $s212
    (block $s211
    (block $s210
    (block $s209
    (block $s208
    (block $s207
    (block $s206
    (block $s205
    (block $s204
    (block $s203
    (block $s202
    (block $s201
    (block $s200
    (block $s199
    (block $s198
    (block $s197
    (block $s196
    (block $s195
    (block $s194
    (block $s193
    (block $s192
    (block $s191
    (block $s190
    (block $s189
    (block $s188
    (block $s187
    (block $s186
    (block $s185
    (block $s184
    (block $s183
    (block $s182
    (block $s181
    (block $s180
    (block $s179
    (block $s178
    (block $s177
    (block $s176
    (block $s175
    (block $s174
    (block $s173
    (block $s172
    (block $s171
    (block $s170
    (block $s169
    (block $s168
    (block $s167
    (block $s166
    (block $s165
    (block $s164
    (block $s163
    (block $s162
    (block $s161
    (block $s160
    (block $s134
    (block $s133
    (block $s122
    (block $s121
    (block $s120
    (block $s119
    (block $s118
    (block $s117
    (block $s116
    (block $s115
    (block $s114
    (block $s113
    (block $s112
    (block $s111
    (block $s110
    (block $s109
    (block $s108
    (block $s107
    (block $s106
    (block $s105
    (block $s104
    (block $s103
    (block $s102
    (block $s101
    (block $s100
    (block $s99
    (block $s98
    (block $s97
    (block $s96
    (block $s95
    (block $s94
    (block $s93
    (block $s92
    (block $s91
    (block $s90
    (block $s89
    (block $s88
    (block $s87
    (block $s86
    (block $s85
    (block $s84
    (block $s83
    (block $s82
    (block $s81
    (block $s80
    (block $s79
    (block $s78
    (block $s77
    (block $s76
    (block $s75
    (block $s74
    (block $s73
    (block $s72
    (block $s71
    (block $s30
    (block $s29
    (block $s16
    (block $s15
      (br_table $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $s15 $s16 $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $s29 $s30 $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $s71 $s72 $s73 $s74 $s75 $s76 $s77 $s78 $s79 $s80 $s81 $s82 $s83 $s84 $s85 $s86 $s87 $s88 $s89 $s90 $s91 $s92 $s93 $s94 $s95 $s96 $s97 $s98 $s99 $s100 $s101 $s102 $s103 $s104 $s105 $s106 $s107 $s108 $s109 $s110 $s111 $s112 $s113 $s114 $s115 $s116 $s117 $s118 $s119 $s120 $s121 $s122 $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $s133 $s134 $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $s160 $s161 $s162 $s163 $s164 $s165 $s166 $s167 $s168 $s169 $s170 $s171 $s172 $s173 $s174 $s175 $s176 $s177 $s178 $s179 $s180 $s181 $s182 $s183 $s184 $s185 $s186 $s187 $s188 $s189 $s190 $s191 $s192 $s193 $s194 $s195 $s196 $s197 $s198 $s199 $s200 $s201 $s202 $s203 $s204 $s205 $s206 $s207 $s208 $s209 $s210 $s211 $s212 $s213 $s214 $s215 $s216 $s217 $s218 $s219 $s220 $s221 $s222 $s223 $s224 $s225 $s226 $s227 $s228 $s229 $s230 $s231 $s232 $s233 $s234 $s235 $s236 $s237 $s238 $s239 $s240 $s241 $s242 $s243 $s244 $s245 $s246 $s247 $s248 $s249 $s250 $s251 $s252 $s253 $s254 $s255 $s256 $s257 $s258 $s259 $s260 $unknown (local.get $id))
    )
    ;; poke_flt64.
    (f64.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $dval (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_flt64.
    (return (call $double (f64.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_flt32.
    (f32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $fval (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_flt32.
    (return (call $float (f32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; calloc.
    (local.set $wide (i64.mul (i64.extend_i32_u (call $ival (call $value_arg (local.get $args) (i32.const 0)))) (i64.extend_i32_u (call $ival (call $value_arg (local.get $args) (i32.const 1))))))
    (if (i64.gt_u (local.get $wide) (i64.const 1073741824)) (then (call $fail (i32.const 72))))
    (return (call $ptr (call $blob_alloc (i32.wrap_i64 (local.get $wide)) (i32.const -1))))
    )
    ;; realloc.
    (local.set $p (call $pval (call $value_arg (local.get $args) (i32.const 0))))
    (local.set $len (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (local.set $q (call $blob_alloc (local.get $len) (i32.const -1)))
    (if (local.get $p)
      (then
        (local.set $block (call $heap_blob_find (local.get $p)))
        (if (i32.eqz (local.get $block)) (then (call $fail (i32.const 75))))
        (local.set $oldlen (i32.load (local.get $block)))
        (memory.copy (local.get $q) (local.get $p)
          (select (local.get $len) (local.get $oldlen)
            (i32.lt_u (local.get $len) (local.get $oldlen))))
        (call $blob_free (local.get $p))))
    (return (call $ptr (local.get $q)))
    )
    ;; free.
    (local.set $p (call $pval (call $value_arg (local.get $args) (i32.const 0))))
    (if (local.get $p) (then (call $blob_free (local.get $p))))
    (return (call $prim (global.get $T_I)))
    )
    ;; addr_free.
    (local.set $p (call $node (global.get $T_FUNPTR)))
    (i32.store offset=4 (local.get $p) (i32.add (global.get $svc_free) (i32.const 1)))
    (return (local.get $p))
    )
    ;; iswindows.
    (return (call $int (i32.const 0)))
    )
    ;; ismacos.
    (return (call $int (i32.const 0)))
    )
    ;; islinux.
    (return (call $int (i32.const 0)))
    )
    ;; malloc.
    (return (call $ptr (call $blob_alloc (call $ival (call $value_arg (local.get $args) (i32.const 0))) (i32.const -1))))
    )
    ;; memcpy.
    (memory.copy (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $pval (call $value_arg (local.get $args) (i32.const 1))) (call $ival (call $value_arg (local.get $args) (i32.const 2))))
    (return (call $prim (global.get $T_I)))
    )
    ;; memmove.
    (memory.copy (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $pval (call $value_arg (local.get $args) (i32.const 1))) (call $ival (call $value_arg (local.get $args) (i32.const 2))))
    (return (call $prim (global.get $T_I)))
    )
    ;; strlen.
    (return (call $int (call $strlen (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; strcpy.
    (local.set $q (call $pval (call $value_arg (local.get $args) (i32.const 1))))
    (memory.copy (call $pval (call $value_arg (local.get $args) (i32.const 0))) (local.get $q) (i32.add (call $strlen (local.get $q)) (i32.const 1)))
    (return (call $prim (global.get $T_I)))
    )
    ;; peekPtr.
    (return (call $ptr (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; peekWord.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; pokePtr.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $pval (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; pokeWord.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_uint8.
    (return (call $int (i32.load8_u align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_uint8.
    (i32.store8 align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_uint16.
    (return (call $int (i32.load16_u align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_uint16.
    (i32.store16 align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_uint32.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_uint32.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_uint64.
    (return (call $int64 (i64.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_uint64.
    (i64.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $i64val (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_uint.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_uint.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_int8.
    (return (call $int (i32.load8_s align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_int8.
    (i32.store8 align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_int16.
    (return (call $int (i32.load16_s align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_int16.
    (i32.store16 align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_int32.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_int32.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_int64.
    (return (call $int64 (i64.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_int64.
    (i64.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $i64val (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_int.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_int.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; peek_llong.
    (return (call $int64 (i64.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; peek_long.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; peek_ullong.
    (return (call $int64 (i64.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; peek_ulong.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; peek_size_t.
    (return (call $int (i32.load align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))))))
    )
    ;; poke_llong.
    (i64.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $i64val (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; poke_long.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; poke_ullong.
    (i64.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $i64val (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; poke_ulong.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; poke_size_t.
    (i32.store align=1 (call $pval (call $value_arg (local.get $args) (i32.const 0))) (call $ival (call $value_arg (local.get $args) (i32.const 1))))
    (return (call $prim (global.get $T_I)))
    )
    ;; sizeof_char.
    (return (call $int (i32.const 1)))
    )
    ;; sizeof_short.
    (return (call $int (i32.const 2)))
    )
    ;; sizeof_int.
    (return (call $int (i32.const 4)))
    )
    ;; sizeof_llong.
    (return (call $int (i32.const 8)))
    )
    ;; sizeof_long.
    (return (call $int (i32.const 4)))
    )
    ;; sizeof_size_t.
    (return (call $int (i32.const 4)))
    )
    ;; want_gmp.
    (return (call $int (i32.const 0)))
    )
    ;; want_imath.
    (return (call $int (i32.const 0)))
    )
    ;; E2BIG.
    (return (call $int (i32.const 1)))
    )
    ;; EACCES.
    (return (call $int (i32.const 2)))
    )
    ;; EADDRINUSE.
    (return (call $int (i32.const 3)))
    )
    ;; EADDRNOTAVAIL.
    (return (call $int (i32.const 4)))
    )
    ;; EADV.
    (return (call $int (i32.const -1)))
    )
    ;; EAFNOSUPPORT.
    (return (call $int (i32.const 5)))
    )
    ;; EAGAIN.
    (return (call $int (i32.const 6)))
    )
    ;; EALREADY.
    (return (call $int (i32.const 7)))
    )
    ;; EBADF.
    (return (call $int (i32.const 8)))
    )
    ;; EBADMSG.
    (return (call $int (i32.const 9)))
    )
    ;; EBADRPC.
    (return (call $int (i32.const -1)))
    )
    ;; EBUSY.
    (return (call $int (i32.const 10)))
    )
    ;; ECHILD.
    (return (call $int (i32.const 12)))
    )
    ;; ECOMM.
    (return (call $int (i32.const -1)))
    )
    ;; ECONNABORTED.
    (return (call $int (i32.const 13)))
    )
    ;; ECONNREFUSED.
    (return (call $int (i32.const 14)))
    )
    ;; ECONNRESET.
    (return (call $int (i32.const 15)))
    )
    ;; EDEADLK.
    (return (call $int (i32.const 16)))
    )
    ;; EDESTADDRREQ.
    (return (call $int (i32.const 17)))
    )
    ;; EDIRTY.
    (return (call $int (i32.const -1)))
    )
    ;; EDOM.
    (return (call $int (i32.const 18)))
    )
    ;; EDQUOT.
    (return (call $int (i32.const 19)))
    )
    ;; EEXIST.
    (return (call $int (i32.const 20)))
    )
    ;; EFAULT.
    (return (call $int (i32.const 21)))
    )
    ;; EFBIG.
    (return (call $int (i32.const 22)))
    )
    ;; EFTYPE.
    (return (call $int (i32.const -1)))
    )
    ;; EHOSTDOWN.
    (return (call $int (i32.const -1)))
    )
    ;; EHOSTUNREACH.
    (return (call $int (i32.const 23)))
    )
    ;; EIDRM.
    (return (call $int (i32.const 24)))
    )
    ;; EILSEQ.
    (return (call $int (i32.const 25)))
    )
    ;; EINPROGRESS.
    (return (call $int (i32.const 26)))
    )
    ;; EINTR.
    (return (call $int (i32.const 27)))
    )
    ;; EINVAL.
    (return (call $int (i32.const 28)))
    )
    ;; EIO.
    (return (call $int (i32.const 29)))
    )
    ;; EISCONN.
    (return (call $int (i32.const 30)))
    )
    ;; EISDIR.
    (return (call $int (i32.const 31)))
    )
    ;; ELOOP.
    (return (call $int (i32.const 32)))
    )
    ;; EMFILE.
    (return (call $int (i32.const 33)))
    )
    ;; EMLINK.
    (return (call $int (i32.const 34)))
    )
    ;; EMSGSIZE.
    (return (call $int (i32.const 35)))
    )
    ;; EMULTIHOP.
    (return (call $int (i32.const 36)))
    )
    ;; ENAMETOOLONG.
    (return (call $int (i32.const 37)))
    )
    ;; ENETDOWN.
    (return (call $int (i32.const 38)))
    )
    ;; ENETRESET.
    (return (call $int (i32.const 39)))
    )
    ;; ENETUNREACH.
    (return (call $int (i32.const 40)))
    )
    ;; ENFILE.
    (return (call $int (i32.const 41)))
    )
    ;; ENOBUFS.
    (return (call $int (i32.const 42)))
    )
    ;; ENODATA.
    (return (call $int (i32.const -1)))
    )
    ;; ENODEV.
    (return (call $int (i32.const 43)))
    )
    ;; ENOENT.
    (return (call $int (i32.const 44)))
    )
    ;; ENOEXEC.
    (return (call $int (i32.const 45)))
    )
    ;; ENOLCK.
    (return (call $int (i32.const 46)))
    )
    ;; ENOLINK.
    (return (call $int (i32.const 47)))
    )
    ;; ENOMEM.
    (return (call $int (i32.const 48)))
    )
    ;; ENOMSG.
    (return (call $int (i32.const 49)))
    )
    ;; ENONET.
    (return (call $int (i32.const -1)))
    )
    ;; ENOPROTOOPT.
    (return (call $int (i32.const 50)))
    )
    ;; ENOSPC.
    (return (call $int (i32.const 51)))
    )
    ;; ENOSR.
    (return (call $int (i32.const -1)))
    )
    ;; ENOSTR.
    (return (call $int (i32.const -1)))
    )
    ;; ENOSYS.
    (return (call $int (i32.const 52)))
    )
    ;; ENOTBLK.
    (return (call $int (i32.const -1)))
    )
    ;; ENOTCONN.
    (return (call $int (i32.const 53)))
    )
    ;; ENOTDIR.
    (return (call $int (i32.const 54)))
    )
    ;; ENOTEMPTY.
    (return (call $int (i32.const 55)))
    )
    ;; ENOTSOCK.
    (return (call $int (i32.const 57)))
    )
    ;; ENOTSUP.
    (return (call $int (i32.const 58)))
    )
    ;; ENOTTY.
    (return (call $int (i32.const 59)))
    )
    ;; ENXIO.
    (return (call $int (i32.const 60)))
    )
    ;; EOPNOTSUPP.
    (return (call $int (i32.const 58)))
    )
    ;; EPERM.
    (return (call $int (i32.const 63)))
    )
    ;; EPFNOSUPPORT.
    (return (call $int (i32.const -1)))
    )
    ;; EPIPE.
    (return (call $int (i32.const 64)))
    )
    ;; EPROCLIM.
    (return (call $int (i32.const -1)))
    )
    ;; EPROCUNAVAIL.
    (return (call $int (i32.const -1)))
    )
    ;; EPROGMISMATCH.
    (return (call $int (i32.const -1)))
    )
    ;; EPROGUNAVAIL.
    (return (call $int (i32.const -1)))
    )
    ;; EPROTO.
    (return (call $int (i32.const 65)))
    )
    ;; EPROTONOSUPPORT.
    (return (call $int (i32.const 66)))
    )
    ;; EPROTOTYPE.
    (return (call $int (i32.const 67)))
    )
    ;; ERANGE.
    (return (call $int (i32.const 68)))
    )
    ;; EREMCHG.
    (return (call $int (i32.const -1)))
    )
    ;; EREMOTE.
    (return (call $int (i32.const -1)))
    )
    ;; EROFS.
    (return (call $int (i32.const 69)))
    )
    ;; ERPCMISMATCH.
    (return (call $int (i32.const -1)))
    )
    ;; ERREMOTE.
    (return (call $int (i32.const -1)))
    )
    ;; ESHUTDOWN.
    (return (call $int (i32.const -1)))
    )
    ;; ESOCKTNOSUPPORT.
    (return (call $int (i32.const -1)))
    )
    ;; ESPIPE.
    (return (call $int (i32.const 70)))
    )
    ;; ESRCH.
    (return (call $int (i32.const 71)))
    )
    ;; ESRMNT.
    (return (call $int (i32.const -1)))
    )
    ;; ESTALE.
    (return (call $int (i32.const 72)))
    )
    ;; ETIME.
    (return (call $int (i32.const -1)))
    )
    ;; ETIMEDOUT.
    (return (call $int (i32.const 73)))
    )
    ;; ETOOMANYREFS.
    (return (call $int (i32.const -1)))
    )
    ;; ETXTBSY.
    (return (call $int (i32.const 74)))
    )
    ;; EUSERS.
    (return (call $int (i32.const -1)))
    )
    ;; EWOULDBLOCK.
    (return (call $int (i32.const 6)))
    )
    ;; EXDEV.
    (return (call $int (i32.const 75)))
    )
    ;; addr_errno.
    (return (call $ptr (i32.const 0x240)))
    )
    ;; strerror_r.
    (local.set $p (call $errno_message (call $ival (call $value_arg (local.get $args) (i32.const 0)))))
    (local.set $len (call $strlen (local.get $p)))
    (if (i32.ge_u (local.get $len) (call $ival (call $value_arg (local.get $args) (i32.const 2))))
      (then (return (call $int (i32.const 68)))))
    (memory.copy (call $pval (call $value_arg (local.get $args) (i32.const 1))) (local.get $p) (i32.add (local.get $len) (i32.const 1)))
    (return (call $int (i32.const 0)))
  )
  (i32.const 0))
