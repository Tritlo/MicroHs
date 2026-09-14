;; Numeric primitives over evaluated scalar nodes.
;; Signed addition, subtraction, multiplication, negation, and quotient raise
;; the same runtime overflow exception as the checked C configuration.
;; Unsigned operations wrap. A shift uses the low five or six count bits,
;; which is the WebAssembly integer-shift rule. Remainder MIN/-1 is zero.
;; Floating-point conversions to Int use saturating WASM conversion semantics.
;; Reinterpretation primitives preserve every payload bit.
;; Return zero only for an unknown tag or a pending Haskell exception.

(func $checked_i64_add (param $x i64) (param $y i64) (result i32)
  (local $r i64)
  (local.set $r (i64.add (local.get $x) (local.get $y)))
  (if (i64.lt_s
        (i64.and (i64.xor (local.get $x) (local.get $r))
                 (i64.xor (local.get $y) (local.get $r))) (i64.const 0))
    (then (return (call $raise_rts (i32.const 7)))))
  (call $int64 (local.get $r)))

(func $checked_i64_sub (param $x i64) (param $y i64) (result i32)
  (local $r i64)
  (local.set $r (i64.sub (local.get $x) (local.get $y)))
  (if (i64.lt_s
        (i64.and (i64.xor (local.get $x) (local.get $y))
                 (i64.xor (local.get $x) (local.get $r))) (i64.const 0))
    (then (return (call $raise_rts (i32.const 7)))))
  (call $int64 (local.get $r)))

;; Check unsigned magnitudes before multiplication. This also handles MIN,
;; whose positive magnitude cannot be represented by a signed i64.
(func $checked_i64_mul (param $x i64) (param $y i64) (result i32)
  (local $a i64) (local $b i64) (local $limit i64)
  (local.set $a (select (i64.sub (i64.const 0) (local.get $x)) (local.get $x)
    (i64.lt_s (local.get $x) (i64.const 0))))
  (local.set $b (select (i64.sub (i64.const 0) (local.get $y)) (local.get $y)
    (i64.lt_s (local.get $y) (i64.const 0))))
  (local.set $limit (select (i64.const -9223372036854775808)
    (i64.const 9223372036854775807)
    (i64.lt_s (i64.xor (local.get $x) (local.get $y)) (i64.const 0))))
  (if (i64.ne (local.get $b) (i64.const 0))
    (then
      (if (i64.gt_u (local.get $a) (i64.div_u (local.get $limit) (local.get $b)))
        (then (return (call $raise_rts (i32.const 7)))))))
  (call $int64 (i64.mul (local.get $x) (local.get $y))))

(func $numeric (param $tag i32) (param $a i32) (param $b i32) (result i32)
  (local $wide i64) (local $result i32)
  (block $unknown

    (block $case220
    (block $case219
    (block $case218
    (block $case217
    (block $case216
    (block $case215
    (block $case214
    (block $case213
    (block $case212
    (block $case211
    (block $case210
    (block $case209
    (block $case208
    (block $case207
    (block $case206
    (block $case205
    (block $case204
    (block $case203
    (block $case202
    (block $case201
    (block $case200
    (block $case199
    (block $case198
    (block $case197
    (block $case196
    (block $case195
    (block $case194
    (block $case193
    (block $case192
    (block $case191
    (block $case190
    (block $case189
    (block $case188
    (block $case173
    (block $case172
    (block $case171
    (block $case170
    (block $case169
    (block $case168
    (block $case167
    (block $case160
    (block $case159
    (block $case158
    (block $case157
    (block $case156
    (block $case155
    (block $case154
    (block $case153
    (block $case152
    (block $case151
    (block $case150
    (block $case149
    (block $case148
    (block $case147
    (block $case146
    (block $case145
    (block $case144
    (block $case143
    (block $case142
    (block $case141
    (block $case140
    (block $case139
    (block $case138
    (block $case137
    (block $case136
    (block $case135
    (block $case134
    (block $case133
    (block $case132
    (block $case131
    (block $case130
    (block $case129
    (block $case128
    (block $case127
    (block $case126
    (block $case125
    (block $case124
    (block $case123
    (block $case122
    (block $case121
    (block $case120
    (block $case119
    (block $case118
    (block $case117
    (block $case116
    (block $case115
    (block $case114
    (block $case113
    (block $case112
    (block $case111
    (block $case110
    (block $case109
    (block $case108
    (block $case107
    (block $case106
    (block $case105
    (block $case104
    (block $case103
    (block $case102
    (block $case101
    (block $case100
    (block $case99
    (block $case98
    (block $case97
    (block $case96
    (block $case95
    (block $case94
    (block $case93
    (block $case92
    (block $case91
    (block $case90
    (block $case89
    (block $case88
    (block $case87
    (block $case86
    (block $case85
      (br_table $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $case85 $case86 $case87 $case88 $case89 $case90 $case91 $case92 $case93 $case94 $case95 $case96 $case97 $case98 $case99 $case100 $case101 $case102 $case103 $case104 $case105 $case106 $case107 $case108 $case109 $case110 $case111 $case112 $case113 $case114 $case115 $case116 $case117 $case118 $case119 $case120 $case121 $case122 $case123 $case124 $case125 $case126 $case127 $case128 $case129 $case130 $case131 $case132 $case133 $case134 $case135 $case136 $case137 $case138 $case139 $case140 $case141 $case142 $case143 $case144 $case145 $case146 $case147 $case148 $case149 $case150 $case151 $case152 $case153 $case154 $case155 $case156 $case157 $case158 $case159 $case160 $unknown $unknown $unknown $unknown $unknown $unknown $case167 $case168 $case169 $case170 $case171 $case172 $case173 $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $unknown $case188 $case189 $case190 $case191 $case192 $case193 $case194 $case195 $case196 $case197 $case198 $case199 $case200 $case201 $case202 $case203 $case204 $case205 $case206 $case207 $case208 $case209 $case210 $case211 $case212 $case213 $case214 $case215 $case216 $case217 $case218 $case219 $case220 $unknown (local.get $tag))
    )
    ;; ADD: serialized tag 85.
    (local.set $wide (i64.add (i64.extend_i32_s (call $ival (local.get $a))) (i64.extend_i32_s (call $ival (local.get $b)))))
    (if (i64.ne (local.get $wide) (i64.extend_i32_s (i32.wrap_i64 (local.get $wide)))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.wrap_i64 (local.get $wide))))
    )
    ;; SUB: serialized tag 86.
    (local.set $wide (i64.sub (i64.extend_i32_s (call $ival (local.get $a))) (i64.extend_i32_s (call $ival (local.get $b)))))
    (if (i64.ne (local.get $wide) (i64.extend_i32_s (i32.wrap_i64 (local.get $wide)))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.wrap_i64 (local.get $wide))))
    )
    ;; MUL: serialized tag 87.
    (local.set $wide (i64.mul (i64.extend_i32_s (call $ival (local.get $a))) (i64.extend_i32_s (call $ival (local.get $b)))))
    (if (i64.ne (local.get $wide) (i64.extend_i32_s (i32.wrap_i64 (local.get $wide)))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.wrap_i64 (local.get $wide))))
    )
    ;; QUOT: serialized tag 88.
    (if (i32.eqz (call $ival (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (if (i32.and (i32.eq (call $ival (local.get $a)) (i32.const -2147483648)) (i32.eq (call $ival (local.get $b)) (i32.const -1))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.div_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; REM: serialized tag 89.
    (if (i32.eqz (call $ival (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int (i32.rem_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; SUBR: serialized tag 90.
    (local.set $wide (i64.sub (i64.extend_i32_s (call $ival (local.get $b))) (i64.extend_i32_s (call $ival (local.get $a)))))
    (if (i64.ne (local.get $wide) (i64.extend_i32_s (i32.wrap_i64 (local.get $wide)))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.wrap_i64 (local.get $wide))))
    )
    ;; NEG: serialized tag 91.
    (if (i32.eq (call $ival (local.get $a)) (i32.const -2147483648)) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int (i32.sub (i32.const 0) (call $ival (local.get $a)))))
    )
    ;; UADD: serialized tag 92.
    (return (call $int (i32.add (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; USUB: serialized tag 93.
    (return (call $int (i32.sub (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; UMUL: serialized tag 94.
    (return (call $int (i32.mul (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; UQUOT: serialized tag 95.
    (if (i32.eqz (call $ival (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int (i32.div_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; UREM: serialized tag 96.
    (if (i32.eqz (call $ival (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int (i32.rem_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; USUBR: serialized tag 97.
    (return (call $int (i32.sub (call $ival (local.get $b)) (call $ival (local.get $a)))))
    )
    ;; UNEG: serialized tag 98.
    (return (call $int (i32.sub (i32.const 0) (call $ival (local.get $a)))))
    )
    ;; AND: serialized tag 99.
    (return (call $int (i32.and (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; OR: serialized tag 100.
    (return (call $int (i32.or (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; XOR: serialized tag 101.
    (return (call $int (i32.xor (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; INV: serialized tag 102.
    (return (call $int (i32.xor (call $ival (local.get $a)) (i32.const -1))))
    )
    ;; SHL: serialized tag 103.
    (return (call $int (i32.shl (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; SHR: serialized tag 104.
    (return (call $int (i32.shr_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; ASHR: serialized tag 105.
    (return (call $int (i32.shr_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; POPCOUNT: serialized tag 106.
    (return (call $int (i32.popcnt (call $ival (local.get $a)))))
    )
    ;; CLZ: serialized tag 107.
    (return (call $int (i32.clz (call $ival (local.get $a)))))
    )
    ;; CTZ: serialized tag 108.
    (return (call $int (i32.ctz (call $ival (local.get $a)))))
    )
    ;; EQ: serialized tag 109.
    (return (call $boolean (i32.eq (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; NE: serialized tag 110.
    (return (call $boolean (i32.ne (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; LT: serialized tag 111.
    (return (call $boolean (i32.lt_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; LE: serialized tag 112.
    (return (call $boolean (i32.le_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; GT: serialized tag 113.
    (return (call $boolean (i32.gt_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; GE: serialized tag 114.
    (return (call $boolean (i32.ge_s (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; ULT: serialized tag 115.
    (return (call $boolean (i32.lt_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; ULE: serialized tag 116.
    (return (call $boolean (i32.le_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; UGT: serialized tag 117.
    (return (call $boolean (i32.gt_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; UGE: serialized tag 118.
    (return (call $boolean (i32.ge_u (call $ival (local.get $a)) (call $ival (local.get $b)))))
    )
    ;; ICMP: serialized tag 119.
    (return (call $ordering (i32.sub (i32.gt_s (call $ival (local.get $a)) (call $ival (local.get $b))) (i32.lt_s (call $ival (local.get $a)) (call $ival (local.get $b))))))
    )
    ;; UCMP: serialized tag 120.
    (return (call $ordering (i32.sub (i32.gt_u (call $ival (local.get $a)) (call $ival (local.get $b))) (i32.lt_u (call $ival (local.get $a)) (call $ival (local.get $b))))))
    )
    ;; ADD64: serialized tag 121.
    (return (call $checked_i64_add (call $i64val (local.get $a)) (call $i64val (local.get $b))))
    )
    ;; SUB64: serialized tag 122.
    (return (call $checked_i64_sub (call $i64val (local.get $a)) (call $i64val (local.get $b))))
    )
    ;; MUL64: serialized tag 123.
    (return (call $checked_i64_mul (call $i64val (local.get $a)) (call $i64val (local.get $b))))
    )
    ;; QUOT64: serialized tag 124.
    (if (i64.eqz (call $i64val (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (if (i32.and (i64.eq (call $i64val (local.get $a)) (i64.const -9223372036854775808)) (i64.eq (call $i64val (local.get $b)) (i64.const -1))) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int64 (i64.div_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; REM64: serialized tag 125.
    (if (i64.eqz (call $i64val (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int64 (i64.rem_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; SUBR64: serialized tag 126.
    (return (call $checked_i64_sub (call $i64val (local.get $b)) (call $i64val (local.get $a))))
    )
    ;; NEG64: serialized tag 127.
    (if (i64.eq (call $i64val (local.get $a)) (i64.const -9223372036854775808)) (then (return (call $raise_rts (i32.const 7)))))
    (return (call $int64 (i64.sub (i64.const 0) (call $i64val (local.get $a)))))
    )
    ;; UADD64: serialized tag 128.
    (return (call $int64 (i64.add (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; USUB64: serialized tag 129.
    (return (call $int64 (i64.sub (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; UMUL64: serialized tag 130.
    (return (call $int64 (i64.mul (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; UQUOT64: serialized tag 131.
    (if (i64.eqz (call $i64val (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int64 (i64.div_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; UREM64: serialized tag 132.
    (if (i64.eqz (call $i64val (local.get $b))) (then (return (call $raise_rts (i32.const 4)))))
    (return (call $int64 (i64.rem_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; USUBR64: serialized tag 133.
    (return (call $int64 (i64.sub (call $i64val (local.get $b)) (call $i64val (local.get $a)))))
    )
    ;; UNEG64: serialized tag 134.
    (return (call $int64 (i64.sub (i64.const 0) (call $i64val (local.get $a)))))
    )
    ;; AND64: serialized tag 135.
    (return (call $int64 (i64.and (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; OR64: serialized tag 136.
    (return (call $int64 (i64.or (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; XOR64: serialized tag 137.
    (return (call $int64 (i64.xor (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; INV64: serialized tag 138.
    (return (call $int64 (i64.xor (call $i64val (local.get $a)) (i64.const -1))))
    )
    ;; SHL64: serialized tag 139.
    (return (call $int64 (i64.shl (call $i64val (local.get $a)) (i64.extend_i32_u (call $ival (local.get $b))))))
    )
    ;; SHR64: serialized tag 140.
    (return (call $int64 (i64.shr_u (call $i64val (local.get $a)) (i64.extend_i32_u (call $ival (local.get $b))))))
    )
    ;; ASHR64: serialized tag 141.
    (return (call $int64 (i64.shr_s (call $i64val (local.get $a)) (i64.extend_i32_u (call $ival (local.get $b))))))
    )
    ;; POPCOUNT64: serialized tag 142.
    (return (call $int (i32.wrap_i64 (i64.popcnt (call $i64val (local.get $a))))))
    )
    ;; CLZ64: serialized tag 143.
    (return (call $int (i32.wrap_i64 (i64.clz (call $i64val (local.get $a))))))
    )
    ;; CTZ64: serialized tag 144.
    (return (call $int (i32.wrap_i64 (i64.ctz (call $i64val (local.get $a))))))
    )
    ;; EQ64: serialized tag 145.
    (return (call $boolean (i64.eq (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; NE64: serialized tag 146.
    (return (call $boolean (i64.ne (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; LT64: serialized tag 147.
    (return (call $boolean (i64.lt_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; LE64: serialized tag 148.
    (return (call $boolean (i64.le_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; GT64: serialized tag 149.
    (return (call $boolean (i64.gt_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; GE64: serialized tag 150.
    (return (call $boolean (i64.ge_s (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; ULT64: serialized tag 151.
    (return (call $boolean (i64.lt_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; ULE64: serialized tag 152.
    (return (call $boolean (i64.le_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; UGT64: serialized tag 153.
    (return (call $boolean (i64.gt_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; UGE64: serialized tag 154.
    (return (call $boolean (i64.ge_u (call $i64val (local.get $a)) (call $i64val (local.get $b)))))
    )
    ;; ICMP64: serialized tag 155.
    (return (call $ordering (i32.sub (i64.gt_s (call $i64val (local.get $a)) (call $i64val (local.get $b))) (i64.lt_s (call $i64val (local.get $a)) (call $i64val (local.get $b))))))
    )
    ;; UCMP64: serialized tag 156.
    (return (call $ordering (i32.sub (i64.gt_u (call $i64val (local.get $a)) (call $i64val (local.get $b))) (i64.lt_u (call $i64val (local.get $a)) (call $i64val (local.get $b))))))
    )
    ;; ITOI64: serialized tag 157.
    (return (call $int64 (i64.extend_i32_s (call $ival (local.get $a)))))
    )
    ;; I64TOI: serialized tag 158.
    (return (call $int (i32.wrap_i64 (call $i64val (local.get $a)))))
    )
    ;; UTOU64: serialized tag 159.
    (return (call $int64 (i64.extend_i32_u (call $ival (local.get $a)))))
    )
    ;; U64TOU: serialized tag 160.
    (return (call $int (i32.wrap_i64 (call $i64val (local.get $a)))))
    )
    ;; TOPTR: serialized tag 167.
    (return (call $ptr (i32.load offset=4 (local.get $a))))
    )
    ;; TOINT: serialized tag 168.
    (return (call $int (i32.load offset=4 (local.get $a))))
    )
    ;; TODBL: serialized tag 169.
    (return (call $double (f64.reinterpret_i64 (call $i64val (local.get $a)))))
    )
    ;; TOFLT: serialized tag 170.
    (return (call $float (f32.reinterpret_i32 (call $ival (local.get $a)))))
    )
    ;; TOFUNPTR: serialized tag 171.
    (local.set $result (call $node (global.get $T_FUNPTR)))
    (i32.store offset=4 (local.get $result) (i32.load offset=4 (local.get $a)))
    (return (local.get $result))
    )
    ;; FROMDBL: serialized tag 172.
    (return (call $int64 (i64.reinterpret_f64 (call $dval (local.get $a)))))
    )
    ;; FROMFLT: serialized tag 173.
    (return (call $int (i32.reinterpret_f32 (call $fval (local.get $a)))))
    )
    ;; ISINT: serialized tag 188.
    ;; The discriminator returns the Int payload, or -1 for another node type.
    (return (call $int (select (i32.load offset=4 (local.get $a)) (i32.const -1)
      (i32.eq (call $tag (local.get $a)) (global.get $T_INT)))))
    )
    ;; FADD: serialized tag 189.
    (return (call $float (f32.add (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FSUB: serialized tag 190.
    (return (call $float (f32.sub (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FMUL: serialized tag 191.
    (return (call $float (f32.mul (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FDIV: serialized tag 192.
    (return (call $float (f32.div (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FNEG: serialized tag 193.
    (return (call $float (f32.neg (call $fval (local.get $a)))))
    )
    ;; ITOF: serialized tag 194.
    (return (call $float (f32.convert_i32_s (call $ival (local.get $a)))))
    )
    ;; I64TOF: serialized tag 195.
    (return (call $float (f32.convert_i64_s (call $i64val (local.get $a)))))
    )
    ;; FTOI: serialized tag 196.
    (return (call $int (i32.trunc_sat_f32_s (call $fval (local.get $a)))))
    )
    ;; UTOF: serialized tag 197.
    (return (call $float (f32.convert_i32_u (call $ival (local.get $a)))))
    )
    ;; FEQ: serialized tag 198.
    (return (call $boolean (f32.eq (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FNE: serialized tag 199.
    (return (call $boolean (f32.ne (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FLT: serialized tag 200.
    (return (call $boolean (f32.lt (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FLE: serialized tag 201.
    (return (call $boolean (f32.le (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FGT: serialized tag 202.
    (return (call $boolean (f32.gt (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; FGE: serialized tag 203.
    (return (call $boolean (f32.ge (call $fval (local.get $a)) (call $fval (local.get $b)))))
    )
    ;; DADD: serialized tag 204.
    (return (call $double (f64.add (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DSUB: serialized tag 205.
    (return (call $double (f64.sub (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DMUL: serialized tag 206.
    (return (call $double (f64.mul (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DDIV: serialized tag 207.
    (return (call $double (f64.div (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DNEG: serialized tag 208.
    (return (call $double (f64.neg (call $dval (local.get $a)))))
    )
    ;; ITOD: serialized tag 209.
    (return (call $double (f64.convert_i32_s (call $ival (local.get $a)))))
    )
    ;; I64TOD: serialized tag 210.
    (return (call $double (f64.convert_i64_s (call $i64val (local.get $a)))))
    )
    ;; DTOI: serialized tag 211.
    (return (call $int (i32.trunc_sat_f64_s (call $dval (local.get $a)))))
    )
    ;; UTOD: serialized tag 212.
    (return (call $double (f64.convert_i32_u (call $ival (local.get $a)))))
    )
    ;; DEQ: serialized tag 213.
    (return (call $boolean (f64.eq (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DNE: serialized tag 214.
    (return (call $boolean (f64.ne (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DLT: serialized tag 215.
    (return (call $boolean (f64.lt (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DLE: serialized tag 216.
    (return (call $boolean (f64.le (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DGT: serialized tag 217.
    (return (call $boolean (f64.gt (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; DGE: serialized tag 218.
    (return (call $boolean (f64.ge (call $dval (local.get $a)) (call $dval (local.get $b)))))
    )
    ;; FTOD: serialized tag 219.
    (return (call $double (f64.promote_f32 (call $fval (local.get $a)))))
    )
    ;; DTOF: serialized tag 220.
    (return (call $float (f32.demote_f64 (call $dval (local.get $a)))))
  )
  (i32.const 0))
