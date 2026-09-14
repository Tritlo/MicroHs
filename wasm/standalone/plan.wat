;; Strictness plans for primitives outside the combinator reducer.
;; Bits 0..3: required argument count, including World for IO operations.
;; Bits 4..11: arguments that must reach WHNF before the primitive runs.
;; Bit 12: force required arguments from right to left. Otherwise use left to
;; right. Arguments are retained in continuation frames during this process.
;; Zero is an unsupported primitive, never a successful no-op.
;; The table occupies 0xc400..0xc663. Values are not heap references.

(data (i32.const 0xc400) "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\32\10\32\10\32\10\32\10\32\10\32\10\11\00\32\10\32\10\32\10\32\10\32\10\32\10\11\00\32\10\32\10\32\10\11\00\32\10\32\10\32\10\11\00\11\00\11\00\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\11\00\32\10\32\10\32\10\32\10\32\10\32\10\11\00\32\10\32\10\32\10\11\00\32\10\32\10\32\10\11\00\11\00\11\00\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\11\00\11\00\11\00\11\00\32\00\11\00\12\00\33\10\32\00\11\00\11\00\11\00\11\00\11\00\11\00\11\00\11\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\11\00\32\10\32\10\32\10\32\10\11\00\11\00\11\00\11\00\11\00\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\11\00\11\00\11\00\11\00\11\00\32\10\32\10\32\10\32\10\32\10\32\10\11\00\11\00\13\00\12\00\12\00\33\10\34\10\33\10\32\00\01\00\12\00\12\00\01\00\03\00\02\00\03\00\33\00\12\00\01\00\01\00\02\00\33\00\03\00\03\00\00\00\12\00\01\00\03\00\22\00\11\00\02\00\01\00\11\00\13\00\01\00\01\00\12\00\13\00\12\00\12\00\13\00\12\00\12\00\12\00\01\00\12\00\12\00\33\00\32\10\32\10\32\10\32\10\32\10\32\10\32\10\32\10\11\00\32\00\11\00\73\00\32\00\33\00\33\00\74\00\12\00\33\00\33\00\11\00\11\00\11\00\32\10\12\00\33\00\02\00\12\00\12\00\04\00\03\00\12\00\12\00\02\00\00\00\00\00\00\00\12\00\12\00\00\00")

(func $primitive_plan (param $tag i32) (param $head i32) (result i32)
  (local $arity i32)
  ;; Runtime services have a dynamic scalar arity and an additional World.
  (if (i32.eq (local.get $tag) (global.get $T_IO_CCALL))
    (then
      (local.set $arity (call $service_arity (i32.load offset=4 (local.get $head))))
      (if (i32.gt_u (local.get $arity) (i32.const 6))
        (then (call $fail (i32.const 27))))
      (return
        (i32.or (i32.add (local.get $arity) (i32.const 1))
          (i32.shl (i32.sub (i32.shl (i32.const 1) (local.get $arity)) (i32.const 1))
            (i32.const 4))))))
  (if (i32.gt_u (local.get $tag) (global.get $T_LAST_TAG))
    (then (return (i32.const 0))))
  (i32.load16_u (i32.add (i32.const 0xc400)
    (i32.shl (local.get $tag) (i32.const 1)))))

;; Review index: tag, primitive, required arguments, strict arguments, order.
;;  85 ADD                    arity=2 strict=0x03 right to left
;;  86 SUB                    arity=2 strict=0x03 right to left
;;  87 MUL                    arity=2 strict=0x03 right to left
;;  88 QUOT                   arity=2 strict=0x03 right to left
;;  89 REM                    arity=2 strict=0x03 right to left
;;  90 SUBR                   arity=2 strict=0x03 right to left
;;  91 NEG                    arity=1 strict=0x01 left to right
;;  92 UADD                   arity=2 strict=0x03 right to left
;;  93 USUB                   arity=2 strict=0x03 right to left
;;  94 UMUL                   arity=2 strict=0x03 right to left
;;  95 UQUOT                  arity=2 strict=0x03 right to left
;;  96 UREM                   arity=2 strict=0x03 right to left
;;  97 USUBR                  arity=2 strict=0x03 right to left
;;  98 UNEG                   arity=1 strict=0x01 left to right
;;  99 AND                    arity=2 strict=0x03 right to left
;; 100 OR                     arity=2 strict=0x03 right to left
;; 101 XOR                    arity=2 strict=0x03 right to left
;; 102 INV                    arity=1 strict=0x01 left to right
;; 103 SHL                    arity=2 strict=0x03 right to left
;; 104 SHR                    arity=2 strict=0x03 right to left
;; 105 ASHR                   arity=2 strict=0x03 right to left
;; 106 POPCOUNT               arity=1 strict=0x01 left to right
;; 107 CLZ                    arity=1 strict=0x01 left to right
;; 108 CTZ                    arity=1 strict=0x01 left to right
;; 109 EQ                     arity=2 strict=0x03 right to left
;; 110 NE                     arity=2 strict=0x03 right to left
;; 111 LT                     arity=2 strict=0x03 right to left
;; 112 LE                     arity=2 strict=0x03 right to left
;; 113 GT                     arity=2 strict=0x03 right to left
;; 114 GE                     arity=2 strict=0x03 right to left
;; 115 ULT                    arity=2 strict=0x03 right to left
;; 116 ULE                    arity=2 strict=0x03 right to left
;; 117 UGT                    arity=2 strict=0x03 right to left
;; 118 UGE                    arity=2 strict=0x03 right to left
;; 119 ICMP                   arity=2 strict=0x03 right to left
;; 120 UCMP                   arity=2 strict=0x03 right to left
;; 121 ADD64                  arity=2 strict=0x03 right to left
;; 122 SUB64                  arity=2 strict=0x03 right to left
;; 123 MUL64                  arity=2 strict=0x03 right to left
;; 124 QUOT64                 arity=2 strict=0x03 right to left
;; 125 REM64                  arity=2 strict=0x03 right to left
;; 126 SUBR64                 arity=2 strict=0x03 right to left
;; 127 NEG64                  arity=1 strict=0x01 left to right
;; 128 UADD64                 arity=2 strict=0x03 right to left
;; 129 USUB64                 arity=2 strict=0x03 right to left
;; 130 UMUL64                 arity=2 strict=0x03 right to left
;; 131 UQUOT64                arity=2 strict=0x03 right to left
;; 132 UREM64                 arity=2 strict=0x03 right to left
;; 133 USUBR64                arity=2 strict=0x03 right to left
;; 134 UNEG64                 arity=1 strict=0x01 left to right
;; 135 AND64                  arity=2 strict=0x03 right to left
;; 136 OR64                   arity=2 strict=0x03 right to left
;; 137 XOR64                  arity=2 strict=0x03 right to left
;; 138 INV64                  arity=1 strict=0x01 left to right
;; 139 SHL64                  arity=2 strict=0x03 right to left
;; 140 SHR64                  arity=2 strict=0x03 right to left
;; 141 ASHR64                 arity=2 strict=0x03 right to left
;; 142 POPCOUNT64             arity=1 strict=0x01 left to right
;; 143 CLZ64                  arity=1 strict=0x01 left to right
;; 144 CTZ64                  arity=1 strict=0x01 left to right
;; 145 EQ64                   arity=2 strict=0x03 right to left
;; 146 NE64                   arity=2 strict=0x03 right to left
;; 147 LT64                   arity=2 strict=0x03 right to left
;; 148 LE64                   arity=2 strict=0x03 right to left
;; 149 GT64                   arity=2 strict=0x03 right to left
;; 150 GE64                   arity=2 strict=0x03 right to left
;; 151 ULT64                  arity=2 strict=0x03 right to left
;; 152 ULE64                  arity=2 strict=0x03 right to left
;; 153 UGT64                  arity=2 strict=0x03 right to left
;; 154 UGE64                  arity=2 strict=0x03 right to left
;; 155 ICMP64                 arity=2 strict=0x03 right to left
;; 156 UCMP64                 arity=2 strict=0x03 right to left
;; 157 ITOI64                 arity=1 strict=0x01 left to right
;; 158 I64TOI                 arity=1 strict=0x01 left to right
;; 159 UTOU64                 arity=1 strict=0x01 left to right
;; 160 U64TOU                 arity=1 strict=0x01 left to right
;; 161 FPADD                  arity=2 strict=0x03 left to right
;; 162 FP2P                   arity=1 strict=0x01 left to right
;; 163 FPNEW                  arity=2 strict=0x01 left to right
;; 164 FPFIN                  arity=3 strict=0x03 right to left
;; 165 FP2BS                  arity=2 strict=0x03 left to right
;; 166 BS2FP                  arity=1 strict=0x01 left to right
;; 167 TOPTR                  arity=1 strict=0x01 left to right
;; 168 TOINT                  arity=1 strict=0x01 left to right
;; 169 TODBL                  arity=1 strict=0x01 left to right
;; 170 TOFLT                  arity=1 strict=0x01 left to right
;; 171 TOFUNPTR               arity=1 strict=0x01 left to right
;; 172 FROMDBL                arity=1 strict=0x01 left to right
;; 173 FROMFLT                arity=1 strict=0x01 left to right
;; 188 ISINT                  arity=1 strict=0x01 left to right
;; 189 FADD                   arity=2 strict=0x03 right to left
;; 190 FSUB                   arity=2 strict=0x03 right to left
;; 191 FMUL                   arity=2 strict=0x03 right to left
;; 192 FDIV                   arity=2 strict=0x03 right to left
;; 193 FNEG                   arity=1 strict=0x01 left to right
;; 194 ITOF                   arity=1 strict=0x01 left to right
;; 195 I64TOF                 arity=1 strict=0x01 left to right
;; 196 FTOI                   arity=1 strict=0x01 left to right
;; 197 UTOF                   arity=1 strict=0x01 left to right
;; 198 FEQ                    arity=2 strict=0x03 right to left
;; 199 FNE                    arity=2 strict=0x03 right to left
;; 200 FLT                    arity=2 strict=0x03 right to left
;; 201 FLE                    arity=2 strict=0x03 right to left
;; 202 FGT                    arity=2 strict=0x03 right to left
;; 203 FGE                    arity=2 strict=0x03 right to left
;; 204 DADD                   arity=2 strict=0x03 right to left
;; 205 DSUB                   arity=2 strict=0x03 right to left
;; 206 DMUL                   arity=2 strict=0x03 right to left
;; 207 DDIV                   arity=2 strict=0x03 right to left
;; 208 DNEG                   arity=1 strict=0x01 left to right
;; 209 ITOD                   arity=1 strict=0x01 left to right
;; 210 I64TOD                 arity=1 strict=0x01 left to right
;; 211 DTOI                   arity=1 strict=0x01 left to right
;; 212 UTOD                   arity=1 strict=0x01 left to right
;; 213 DEQ                    arity=2 strict=0x03 right to left
;; 214 DNE                    arity=2 strict=0x03 right to left
;; 215 DLT                    arity=2 strict=0x03 right to left
;; 216 DLE                    arity=2 strict=0x03 right to left
;; 217 DGT                    arity=2 strict=0x03 right to left
;; 218 DGE                    arity=2 strict=0x03 right to left
;; 219 FTOD                   arity=1 strict=0x01 left to right
;; 220 DTOF                   arity=1 strict=0x01 left to right
;; 221 ARR_ALLOC              arity=3 strict=0x01 left to right
;; 222 ARR_COPY               arity=2 strict=0x01 left to right
;; 223 ARR_SIZE               arity=2 strict=0x01 left to right
;; 224 ARR_READ               arity=3 strict=0x03 right to left
;; 225 ARR_WRITE              arity=4 strict=0x03 right to left
;; 226 ARR_TRUNC              arity=3 strict=0x03 right to left
;; 227 ARR_EQ                 arity=2 strict=0x03 left to right
;; 228 RAISE                  arity=1 strict=0x00 left to right
;; 229 SEQ                    arity=2 strict=0x01 left to right
;; 230 RNF                    arity=2 strict=0x01 left to right
;; 231 TICK                   arity=1 strict=0x00 left to right
;; 232 IO_BIND                arity=3 strict=0x00 left to right
;; 233 IO_THEN                arity=2 strict=0x00 left to right
;; 234 IO_RETURN              arity=3 strict=0x00 left to right
;; 235 IO_SERIALIZE           arity=3 strict=0x03 left to right
;; 236 IO_DESERIALIZE         arity=2 strict=0x01 left to right
;; 237 IO_GETARGREF           arity=1 strict=0x00 left to right
;; 238 IO_PERFORMIO           arity=1 strict=0x00 left to right
;; 239 IO_ATOMIC              arity=2 strict=0x00 left to right
;; 240 IO_PRINT               arity=3 strict=0x03 left to right
;; 241 CATCH                  arity=3 strict=0x00 left to right
;; 242 CATCHR                 arity=3 strict=0x00 left to right
;; 244 IO_GC                  arity=2 strict=0x01 left to right
;; 245 IO_STATS               arity=1 strict=0x00 left to right
;; 246 IO_LAZYBIND            arity=3 strict=0x00 left to right
;; 247 IO_STRICT              arity=2 strict=0x02 left to right
;; 248 DYNSYM                 arity=1 strict=0x01 left to right
;; 249 IO_FORK                arity=2 strict=0x00 left to right
;; 250 IO_THID                arity=1 strict=0x00 left to right
;; 251 THNUM                  arity=1 strict=0x01 left to right
;; 252 IO_THROWTO             arity=3 strict=0x01 left to right
;; 253 IO_YIELD               arity=1 strict=0x00 left to right
;; 254 IO_NEWMVAR             arity=1 strict=0x00 left to right
;; 255 IO_TAKEMVAR            arity=2 strict=0x01 left to right
;; 256 IO_PUTMVAR             arity=3 strict=0x01 left to right
;; 257 IO_READMVAR            arity=2 strict=0x01 left to right
;; 258 IO_TRYTAKEMVAR         arity=2 strict=0x01 left to right
;; 259 IO_TRYPUTMVAR          arity=3 strict=0x01 left to right
;; 260 IO_TRYREADMVAR         arity=2 strict=0x01 left to right
;; 261 IO_THREADDELAY         arity=2 strict=0x01 left to right
;; 262 IO_THREADSTATUS        arity=2 strict=0x01 left to right
;; 263 IO_GETMASKINGSTATE     arity=1 strict=0x00 left to right
;; 264 IO_SETMASKINGSTATE     arity=2 strict=0x01 left to right
;; 265 PACKCSTRING            arity=2 strict=0x01 left to right
;; 266 PACKCSTRINGLEN         arity=3 strict=0x03 left to right
;; 267 BSAPPEND               arity=2 strict=0x03 right to left
;; 268 BSEQ                   arity=2 strict=0x03 right to left
;; 269 BSNE                   arity=2 strict=0x03 right to left
;; 270 BSLT                   arity=2 strict=0x03 right to left
;; 271 BSLE                   arity=2 strict=0x03 right to left
;; 272 BSGT                   arity=2 strict=0x03 right to left
;; 273 BSGE                   arity=2 strict=0x03 right to left
;; 274 BSCMP                  arity=2 strict=0x03 right to left
;; 275 BSUNPACK               arity=1 strict=0x01 left to right
;; 276 BSREPLICATE            arity=2 strict=0x03 left to right
;; 277 BSLENGTH               arity=1 strict=0x01 left to right
;; 278 BSSUBSTR               arity=3 strict=0x07 left to right
;; 279 BSINDEX                arity=2 strict=0x03 left to right
;; 280 BSNEW                  arity=3 strict=0x03 left to right
;; 281 BSREAD                 arity=3 strict=0x03 left to right
;; 282 BSWRITE                arity=4 strict=0x07 left to right
;; 283 BSFREEZE               arity=2 strict=0x01 left to right
;; 284 BSAPPBYTE              arity=3 strict=0x03 left to right
;; 285 BSAPPCHAR              arity=3 strict=0x03 left to right
;; 286 BSFROMUTF8             arity=1 strict=0x01 left to right
;; 287 BSHEADUTF8             arity=1 strict=0x01 left to right
;; 288 BSTAILUTF8             arity=1 strict=0x01 left to right
;; 289 BSAPPENDDOT            arity=2 strict=0x03 right to left
;; 290 BSGRAB                 arity=2 strict=0x01 left to right
;; 291 BSGRABLEN              arity=3 strict=0x03 left to right
;; 292 SPNEW                  arity=2 strict=0x00 left to right
;; 293 SPDEREF                arity=2 strict=0x01 left to right
;; 294 SPFREE                 arity=2 strict=0x01 left to right
;; 295 WKNEWFIN               arity=4 strict=0x00 left to right
;; 296 WKNEW                  arity=3 strict=0x00 left to right
;; 297 WKDEREF                arity=2 strict=0x01 left to right
;; 298 WKFINAL                arity=2 strict=0x01 left to right
;; 299 IO_PP                  arity=2 strict=0x00 left to right
;; 303 IO_WAITRDFD            arity=2 strict=0x01 left to right
;; 304 IO_WAITWRFD            arity=2 strict=0x01 left to right
