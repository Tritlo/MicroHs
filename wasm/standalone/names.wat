;; Serialized tags and names are part of the v8.4 bytecode contract.
;; These checked-in tables include all primitive aliases and fixed services.
;; The standalone build reads this file. It does not read C source.
;; A service name identifies an operation. It is not a C symbol address.
;; Hash slots contain a one-based record index. Zero marks an empty slot.
;; Records contain name address, byte length, and tag or scalar arity.

(global $T_FREE i32 (i32.const 0))
(global $T_IND i32 (i32.const 1))
(global $T_AP i32 (i32.const 2))
(global $T_INT i32 (i32.const 3))
(global $T_INT64 i32 (i32.const 4))
(global $T_DBL i32 (i32.const 5))
(global $T_FLT32 i32 (i32.const 6))
(global $T_PTR i32 (i32.const 7))
(global $T_FUNPTR i32 (i32.const 8))
(global $T_FORPTR i32 (i32.const 9))
(global $T_BADDYN i32 (i32.const 10))
(global $T_ARR i32 (i32.const 11))
(global $T_THID i32 (i32.const 12))
(global $T_MVAR i32 (i32.const 13))
(global $T_WEAK i32 (i32.const 14))
(global $T_S i32 (i32.const 15))
(global $T_K i32 (i32.const 16))
(global $T_I i32 (i32.const 17))
(global $T_B i32 (i32.const 18))
(global $T_C i32 (i32.const 19))
(global $T_A i32 (i32.const 20))
(global $T_Y i32 (i32.const 21))
(global $T_SS i32 (i32.const 22))
(global $T_BB i32 (i32.const 23))
(global $T_CC i32 (i32.const 24))
(global $T_P i32 (i32.const 25))
(global $T_R i32 (i32.const 26))
(global $T_O i32 (i32.const 27))
(global $T_U i32 (i32.const 28))
(global $T_Z i32 (i32.const 29))
(global $T_J i32 (i32.const 30))
(global $T_K2 i32 (i32.const 31))
(global $T_K3 i32 (i32.const 32))
(global $T_K4 i32 (i32.const 33))
(global $T_CCB i32 (i32.const 34))
(global $T_L i32 (i32.const 35))
(global $T_KK i32 (i32.const 36))
(global $T_KA i32 (i32.const 37))
(global $T_T3 i32 (i32.const 38))
(global $T_T4 i32 (i32.const 39))
(global $T_T5 i32 (i32.const 40))
(global $T_T6 i32 (i32.const 41))
(global $T_T7 i32 (i32.const 42))
(global $T_T8 i32 (i32.const 43))
(global $T_T9 i32 (i32.const 44))
(global $T_T10 i32 (i32.const 45))
(global $T_T11 i32 (i32.const 46))
(global $T_T12 i32 (i32.const 47))
(global $T_T13 i32 (i32.const 48))
(global $T_T14 i32 (i32.const 49))
(global $T_T15 i32 (i32.const 50))
(global $T_T16 i32 (i32.const 51))
(global $T_TAG0 i32 (i32.const 52))
(global $T_TAG1 i32 (i32.const 53))
(global $T_TAG2 i32 (i32.const 54))
(global $T_TAG3 i32 (i32.const 55))
(global $T_TAG4 i32 (i32.const 56))
(global $T_TAG5 i32 (i32.const 57))
(global $T_TAG6 i32 (i32.const 58))
(global $T_TAG7 i32 (i32.const 59))
(global $T_TAG8 i32 (i32.const 60))
(global $T_TAG9 i32 (i32.const 61))
(global $T_TAG10 i32 (i32.const 62))
(global $T_TAG11 i32 (i32.const 63))
(global $T_TAG12 i32 (i32.const 64))
(global $T_TAG13 i32 (i32.const 65))
(global $T_TAG14 i32 (i32.const 66))
(global $T_TAG15 i32 (i32.const 67))
(global $T_TAG16 i32 (i32.const 68))
(global $T_TAG17 i32 (i32.const 69))
(global $T_TAG18 i32 (i32.const 70))
(global $T_TAG19 i32 (i32.const 71))
(global $T_TAG20 i32 (i32.const 72))
(global $T_TAG21 i32 (i32.const 73))
(global $T_TAG22 i32 (i32.const 74))
(global $T_TAG23 i32 (i32.const 75))
(global $T_TAG24 i32 (i32.const 76))
(global $T_TAG25 i32 (i32.const 77))
(global $T_TAG26 i32 (i32.const 78))
(global $T_TAG27 i32 (i32.const 79))
(global $T_TAG28 i32 (i32.const 80))
(global $T_TAG29 i32 (i32.const 81))
(global $T_TAG30 i32 (i32.const 82))
(global $T_TAG31 i32 (i32.const 83))
(global $T_TAG32 i32 (i32.const 84))
(global $T_ADD i32 (i32.const 85))
(global $T_SUB i32 (i32.const 86))
(global $T_MUL i32 (i32.const 87))
(global $T_QUOT i32 (i32.const 88))
(global $T_REM i32 (i32.const 89))
(global $T_SUBR i32 (i32.const 90))
(global $T_NEG i32 (i32.const 91))
(global $T_UADD i32 (i32.const 92))
(global $T_USUB i32 (i32.const 93))
(global $T_UMUL i32 (i32.const 94))
(global $T_UQUOT i32 (i32.const 95))
(global $T_UREM i32 (i32.const 96))
(global $T_USUBR i32 (i32.const 97))
(global $T_UNEG i32 (i32.const 98))
(global $T_AND i32 (i32.const 99))
(global $T_OR i32 (i32.const 100))
(global $T_XOR i32 (i32.const 101))
(global $T_INV i32 (i32.const 102))
(global $T_SHL i32 (i32.const 103))
(global $T_SHR i32 (i32.const 104))
(global $T_ASHR i32 (i32.const 105))
(global $T_POPCOUNT i32 (i32.const 106))
(global $T_CLZ i32 (i32.const 107))
(global $T_CTZ i32 (i32.const 108))
(global $T_EQ i32 (i32.const 109))
(global $T_NE i32 (i32.const 110))
(global $T_LT i32 (i32.const 111))
(global $T_LE i32 (i32.const 112))
(global $T_GT i32 (i32.const 113))
(global $T_GE i32 (i32.const 114))
(global $T_ULT i32 (i32.const 115))
(global $T_ULE i32 (i32.const 116))
(global $T_UGT i32 (i32.const 117))
(global $T_UGE i32 (i32.const 118))
(global $T_ICMP i32 (i32.const 119))
(global $T_UCMP i32 (i32.const 120))
(global $T_ADD64 i32 (i32.const 121))
(global $T_SUB64 i32 (i32.const 122))
(global $T_MUL64 i32 (i32.const 123))
(global $T_QUOT64 i32 (i32.const 124))
(global $T_REM64 i32 (i32.const 125))
(global $T_SUBR64 i32 (i32.const 126))
(global $T_NEG64 i32 (i32.const 127))
(global $T_UADD64 i32 (i32.const 128))
(global $T_USUB64 i32 (i32.const 129))
(global $T_UMUL64 i32 (i32.const 130))
(global $T_UQUOT64 i32 (i32.const 131))
(global $T_UREM64 i32 (i32.const 132))
(global $T_USUBR64 i32 (i32.const 133))
(global $T_UNEG64 i32 (i32.const 134))
(global $T_AND64 i32 (i32.const 135))
(global $T_OR64 i32 (i32.const 136))
(global $T_XOR64 i32 (i32.const 137))
(global $T_INV64 i32 (i32.const 138))
(global $T_SHL64 i32 (i32.const 139))
(global $T_SHR64 i32 (i32.const 140))
(global $T_ASHR64 i32 (i32.const 141))
(global $T_POPCOUNT64 i32 (i32.const 142))
(global $T_CLZ64 i32 (i32.const 143))
(global $T_CTZ64 i32 (i32.const 144))
(global $T_EQ64 i32 (i32.const 145))
(global $T_NE64 i32 (i32.const 146))
(global $T_LT64 i32 (i32.const 147))
(global $T_LE64 i32 (i32.const 148))
(global $T_GT64 i32 (i32.const 149))
(global $T_GE64 i32 (i32.const 150))
(global $T_ULT64 i32 (i32.const 151))
(global $T_ULE64 i32 (i32.const 152))
(global $T_UGT64 i32 (i32.const 153))
(global $T_UGE64 i32 (i32.const 154))
(global $T_ICMP64 i32 (i32.const 155))
(global $T_UCMP64 i32 (i32.const 156))
(global $T_ITOI64 i32 (i32.const 157))
(global $T_I64TOI i32 (i32.const 158))
(global $T_UTOU64 i32 (i32.const 159))
(global $T_U64TOU i32 (i32.const 160))
(global $T_FPADD i32 (i32.const 161))
(global $T_FP2P i32 (i32.const 162))
(global $T_FPNEW i32 (i32.const 163))
(global $T_FPFIN i32 (i32.const 164))
(global $T_FP2BS i32 (i32.const 165))
(global $T_BS2FP i32 (i32.const 166))
(global $T_TOPTR i32 (i32.const 167))
(global $T_TOINT i32 (i32.const 168))
(global $T_TODBL i32 (i32.const 169))
(global $T_TOFLT i32 (i32.const 170))
(global $T_TOFUNPTR i32 (i32.const 171))
(global $T_FROMDBL i32 (i32.const 172))
(global $T_FROMFLT i32 (i32.const 173))
(global $T_BININT2 i32 (i32.const 174))
(global $T_BININT1 i32 (i32.const 175))
(global $T_UNINT1 i32 (i32.const 176))
(global $T_BININT64_2 i32 (i32.const 177))
(global $T_BININT64_1 i32 (i32.const 178))
(global $T_UNINT64_1 i32 (i32.const 179))
(global $T_BINFLT2 i32 (i32.const 180))
(global $T_BINFLT1 i32 (i32.const 181))
(global $T_UNFLT1 i32 (i32.const 182))
(global $T_BINDBL2 i32 (i32.const 183))
(global $T_BINDBL1 i32 (i32.const 184))
(global $T_UNDBL1 i32 (i32.const 185))
(global $T_BINBS2 i32 (i32.const 186))
(global $T_BINBS1 i32 (i32.const 187))
(global $T_ISINT i32 (i32.const 188))
(global $T_FADD i32 (i32.const 189))
(global $T_FSUB i32 (i32.const 190))
(global $T_FMUL i32 (i32.const 191))
(global $T_FDIV i32 (i32.const 192))
(global $T_FNEG i32 (i32.const 193))
(global $T_ITOF i32 (i32.const 194))
(global $T_I64TOF i32 (i32.const 195))
(global $T_FTOI i32 (i32.const 196))
(global $T_UTOF i32 (i32.const 197))
(global $T_FEQ i32 (i32.const 198))
(global $T_FNE i32 (i32.const 199))
(global $T_FLT i32 (i32.const 200))
(global $T_FLE i32 (i32.const 201))
(global $T_FGT i32 (i32.const 202))
(global $T_FGE i32 (i32.const 203))
(global $T_DADD i32 (i32.const 204))
(global $T_DSUB i32 (i32.const 205))
(global $T_DMUL i32 (i32.const 206))
(global $T_DDIV i32 (i32.const 207))
(global $T_DNEG i32 (i32.const 208))
(global $T_ITOD i32 (i32.const 209))
(global $T_I64TOD i32 (i32.const 210))
(global $T_DTOI i32 (i32.const 211))
(global $T_UTOD i32 (i32.const 212))
(global $T_DEQ i32 (i32.const 213))
(global $T_DNE i32 (i32.const 214))
(global $T_DLT i32 (i32.const 215))
(global $T_DLE i32 (i32.const 216))
(global $T_DGT i32 (i32.const 217))
(global $T_DGE i32 (i32.const 218))
(global $T_FTOD i32 (i32.const 219))
(global $T_DTOF i32 (i32.const 220))
(global $T_ARR_ALLOC i32 (i32.const 221))
(global $T_ARR_COPY i32 (i32.const 222))
(global $T_ARR_SIZE i32 (i32.const 223))
(global $T_ARR_READ i32 (i32.const 224))
(global $T_ARR_WRITE i32 (i32.const 225))
(global $T_ARR_TRUNC i32 (i32.const 226))
(global $T_ARR_EQ i32 (i32.const 227))
(global $T_RAISE i32 (i32.const 228))
(global $T_SEQ i32 (i32.const 229))
(global $T_RNF i32 (i32.const 230))
(global $T_TICK i32 (i32.const 231))
(global $T_IO_BIND i32 (i32.const 232))
(global $T_IO_THEN i32 (i32.const 233))
(global $T_IO_RETURN i32 (i32.const 234))
(global $T_IO_SERIALIZE i32 (i32.const 235))
(global $T_IO_DESERIALIZE i32 (i32.const 236))
(global $T_IO_GETARGREF i32 (i32.const 237))
(global $T_IO_PERFORMIO i32 (i32.const 238))
(global $T_IO_ATOMIC i32 (i32.const 239))
(global $T_IO_PRINT i32 (i32.const 240))
(global $T_CATCH i32 (i32.const 241))
(global $T_CATCHR i32 (i32.const 242))
(global $T_IO_CCALL i32 (i32.const 243))
(global $T_IO_GC i32 (i32.const 244))
(global $T_IO_STATS i32 (i32.const 245))
(global $T_IO_LAZYBIND i32 (i32.const 246))
(global $T_IO_STRICT i32 (i32.const 247))
(global $T_DYNSYM i32 (i32.const 248))
(global $T_IO_FORK i32 (i32.const 249))
(global $T_IO_THID i32 (i32.const 250))
(global $T_THNUM i32 (i32.const 251))
(global $T_IO_THROWTO i32 (i32.const 252))
(global $T_IO_YIELD i32 (i32.const 253))
(global $T_IO_NEWMVAR i32 (i32.const 254))
(global $T_IO_TAKEMVAR i32 (i32.const 255))
(global $T_IO_PUTMVAR i32 (i32.const 256))
(global $T_IO_READMVAR i32 (i32.const 257))
(global $T_IO_TRYTAKEMVAR i32 (i32.const 258))
(global $T_IO_TRYPUTMVAR i32 (i32.const 259))
(global $T_IO_TRYREADMVAR i32 (i32.const 260))
(global $T_IO_THREADDELAY i32 (i32.const 261))
(global $T_IO_THREADSTATUS i32 (i32.const 262))
(global $T_IO_GETMASKINGSTATE i32 (i32.const 263))
(global $T_IO_SETMASKINGSTATE i32 (i32.const 264))
(global $T_PACKCSTRING i32 (i32.const 265))
(global $T_PACKCSTRINGLEN i32 (i32.const 266))
(global $T_BSAPPEND i32 (i32.const 267))
(global $T_BSEQ i32 (i32.const 268))
(global $T_BSNE i32 (i32.const 269))
(global $T_BSLT i32 (i32.const 270))
(global $T_BSLE i32 (i32.const 271))
(global $T_BSGT i32 (i32.const 272))
(global $T_BSGE i32 (i32.const 273))
(global $T_BSCMP i32 (i32.const 274))
(global $T_BSUNPACK i32 (i32.const 275))
(global $T_BSREPLICATE i32 (i32.const 276))
(global $T_BSLENGTH i32 (i32.const 277))
(global $T_BSSUBSTR i32 (i32.const 278))
(global $T_BSINDEX i32 (i32.const 279))
(global $T_BSNEW i32 (i32.const 280))
(global $T_BSREAD i32 (i32.const 281))
(global $T_BSWRITE i32 (i32.const 282))
(global $T_BSFREEZE i32 (i32.const 283))
(global $T_BSAPPBYTE i32 (i32.const 284))
(global $T_BSAPPCHAR i32 (i32.const 285))
(global $T_BSFROMUTF8 i32 (i32.const 286))
(global $T_BSHEADUTF8 i32 (i32.const 287))
(global $T_BSTAILUTF8 i32 (i32.const 288))
(global $T_BSAPPENDDOT i32 (i32.const 289))
(global $T_BSGRAB i32 (i32.const 290))
(global $T_BSGRABLEN i32 (i32.const 291))
(global $T_SPNEW i32 (i32.const 292))
(global $T_SPDEREF i32 (i32.const 293))
(global $T_SPFREE i32 (i32.const 294))
(global $T_WKNEWFIN i32 (i32.const 295))
(global $T_WKNEW i32 (i32.const 296))
(global $T_WKDEREF i32 (i32.const 297))
(global $T_WKFINAL i32 (i32.const 298))
(global $T_IO_PP i32 (i32.const 299))
(global $T_IO_STDIN i32 (i32.const 300))
(global $T_IO_STDOUT i32 (i32.const 301))
(global $T_IO_STDERR i32 (i32.const 302))
(global $T_IO_WAITRDFD i32 (i32.const 303))
(global $T_IO_WAITWRFD i32 (i32.const 304))
(global $T_LAST_TAG i32 (i32.const 305))

(global $service_count i32 (i32.const 284))
(global $svc_GETRAW i32 (i32.const 0)) ;; 0 arguments
(global $svc_GETTIMEMICRO i32 (i32.const 1)) ;; 0 arguments
(global $svc_GETBOOTTIMEMICRO i32 (i32.const 2)) ;; 0 arguments
(global $svc_acos i32 (i32.const 3)) ;; 1 arguments
(global $svc_asin i32 (i32.const 4)) ;; 1 arguments
(global $svc_atan i32 (i32.const 5)) ;; 1 arguments
(global $svc_atan2 i32 (i32.const 6)) ;; 2 arguments
(global $svc_cos i32 (i32.const 7)) ;; 1 arguments
(global $svc_exp i32 (i32.const 8)) ;; 1 arguments
(global $svc_log i32 (i32.const 9)) ;; 1 arguments
(global $svc_sin i32 (i32.const 10)) ;; 1 arguments
(global $svc_sqrt i32 (i32.const 11)) ;; 1 arguments
(global $svc_tan i32 (i32.const 12)) ;; 1 arguments
(global $svc_scalbn i32 (i32.const 13)) ;; 2 arguments
(global $svc_pow i32 (i32.const 14)) ;; 2 arguments
(global $svc_poke_flt64 i32 (i32.const 15)) ;; 2 arguments
(global $svc_peek_flt64 i32 (i32.const 16)) ;; 1 arguments
(global $svc_acosf i32 (i32.const 17)) ;; 1 arguments
(global $svc_asinf i32 (i32.const 18)) ;; 1 arguments
(global $svc_atanf i32 (i32.const 19)) ;; 1 arguments
(global $svc_atan2f i32 (i32.const 20)) ;; 2 arguments
(global $svc_cosf i32 (i32.const 21)) ;; 1 arguments
(global $svc_expf i32 (i32.const 22)) ;; 1 arguments
(global $svc_logf i32 (i32.const 23)) ;; 1 arguments
(global $svc_sinf i32 (i32.const 24)) ;; 1 arguments
(global $svc_sqrtf i32 (i32.const 25)) ;; 1 arguments
(global $svc_tanf i32 (i32.const 26)) ;; 1 arguments
(global $svc_scalbnf i32 (i32.const 27)) ;; 2 arguments
(global $svc_powf i32 (i32.const 28)) ;; 2 arguments
(global $svc_poke_flt32 i32 (i32.const 29)) ;; 2 arguments
(global $svc_peek_flt32 i32 (i32.const 30)) ;; 1 arguments
(global $svc_js_debug i32 (i32.const 31)) ;; 1 arguments
(global $svc_js_eval_run i32 (i32.const 32)) ;; 1 arguments
(global $svc_js_eval_call i32 (i32.const 33)) ;; 1 arguments
(global $svc_js_set_haskellCallback i32 (i32.const 34)) ;; 1 arguments
(global $svc_add_FILE i32 (i32.const 35)) ;; 1 arguments
(global $svc_putchar i32 (i32.const 36)) ;; 1 arguments
(global $svc_fopen i32 (i32.const 37)) ;; 2 arguments
(global $svc_tmpname i32 (i32.const 38)) ;; 2 arguments
(global $svc_remove i32 (i32.const 39)) ;; 1 arguments
(global $svc_system i32 (i32.const 40)) ;; 1 arguments
(global $svc_add_fd i32 (i32.const 41)) ;; 1 arguments
(global $svc_open i32 (i32.const 42)) ;; 3 arguments
(global $svc_add_buf i32 (i32.const 43)) ;; 2 arguments
(global $svc_add_crlf i32 (i32.const 44)) ;; 1 arguments
(global $svc_add_utf8 i32 (i32.const 45)) ;; 1 arguments
(global $svc_add_base64_encoder i32 (i32.const 46)) ;; 1 arguments
(global $svc_add_base64_decoder i32 (i32.const 47)) ;; 1 arguments
(global $svc_closeb i32 (i32.const 48)) ;; 1 arguments
(global $svc_addr_closeb i32 (i32.const 49)) ;; 0 arguments
(global $svc_flushb i32 (i32.const 50)) ;; 1 arguments
(global $svc_getb i32 (i32.const 51)) ;; 1 arguments
(global $svc_putb i32 (i32.const 52)) ;; 2 arguments
(global $svc_ungetb i32 (i32.const 53)) ;; 2 arguments
(global $svc_openb_wr_mem i32 (i32.const 54)) ;; 0 arguments
(global $svc_openb_rd_mem i32 (i32.const 55)) ;; 2 arguments
(global $svc_get_mem i32 (i32.const 56)) ;; 3 arguments
(global $svc_readb i32 (i32.const 57)) ;; 3 arguments
(global $svc_writeb i32 (i32.const 58)) ;; 3 arguments
(global $svc_md5Array i32 (i32.const 59)) ;; 3 arguments
(global $svc_md5BFILE i32 (i32.const 60)) ;; 2 arguments
(global $svc_md5String i32 (i32.const 61)) ;; 2 arguments
(global $svc_add_lz77_compressor i32 (i32.const 62)) ;; 1 arguments
(global $svc_add_lz77_decompressor i32 (i32.const 63)) ;; 1 arguments
(global $svc_lz77c i32 (i32.const 64)) ;; 3 arguments
(global $svc_add_lzma_compressor i32 (i32.const 65)) ;; 1 arguments
(global $svc_add_lzma_decompressor i32 (i32.const 66)) ;; 1 arguments
(global $svc_add_rle_compressor i32 (i32.const 67)) ;; 1 arguments
(global $svc_add_rle_decompressor i32 (i32.const 68)) ;; 1 arguments
(global $svc_add_bwt_compressor i32 (i32.const 69)) ;; 1 arguments
(global $svc_add_bwt_decompressor i32 (i32.const 70)) ;; 1 arguments
(global $svc_calloc i32 (i32.const 71)) ;; 2 arguments
(global $svc_realloc i32 (i32.const 72)) ;; 2 arguments
(global $svc_free i32 (i32.const 73)) ;; 1 arguments
(global $svc_addr_free i32 (i32.const 74)) ;; 0 arguments
(global $svc_iswindows i32 (i32.const 75)) ;; 0 arguments
(global $svc_ismacos i32 (i32.const 76)) ;; 0 arguments
(global $svc_islinux i32 (i32.const 77)) ;; 0 arguments
(global $svc_malloc i32 (i32.const 78)) ;; 1 arguments
(global $svc_memcpy i32 (i32.const 79)) ;; 3 arguments
(global $svc_memmove i32 (i32.const 80)) ;; 3 arguments
(global $svc_strlen i32 (i32.const 81)) ;; 1 arguments
(global $svc_strcpy i32 (i32.const 82)) ;; 2 arguments
(global $svc_peekPtr i32 (i32.const 83)) ;; 1 arguments
(global $svc_peekWord i32 (i32.const 84)) ;; 1 arguments
(global $svc_pokePtr i32 (i32.const 85)) ;; 2 arguments
(global $svc_pokeWord i32 (i32.const 86)) ;; 2 arguments
(global $svc_peek_uint8 i32 (i32.const 87)) ;; 1 arguments
(global $svc_poke_uint8 i32 (i32.const 88)) ;; 2 arguments
(global $svc_peek_uint16 i32 (i32.const 89)) ;; 1 arguments
(global $svc_poke_uint16 i32 (i32.const 90)) ;; 2 arguments
(global $svc_peek_uint32 i32 (i32.const 91)) ;; 1 arguments
(global $svc_poke_uint32 i32 (i32.const 92)) ;; 2 arguments
(global $svc_peek_uint64 i32 (i32.const 93)) ;; 1 arguments
(global $svc_poke_uint64 i32 (i32.const 94)) ;; 2 arguments
(global $svc_peek_uint i32 (i32.const 95)) ;; 1 arguments
(global $svc_poke_uint i32 (i32.const 96)) ;; 2 arguments
(global $svc_peek_int8 i32 (i32.const 97)) ;; 1 arguments
(global $svc_poke_int8 i32 (i32.const 98)) ;; 2 arguments
(global $svc_peek_int16 i32 (i32.const 99)) ;; 1 arguments
(global $svc_poke_int16 i32 (i32.const 100)) ;; 2 arguments
(global $svc_peek_int32 i32 (i32.const 101)) ;; 1 arguments
(global $svc_poke_int32 i32 (i32.const 102)) ;; 2 arguments
(global $svc_peek_int64 i32 (i32.const 103)) ;; 1 arguments
(global $svc_poke_int64 i32 (i32.const 104)) ;; 2 arguments
(global $svc_peek_int i32 (i32.const 105)) ;; 1 arguments
(global $svc_poke_int i32 (i32.const 106)) ;; 2 arguments
(global $svc_peek_llong i32 (i32.const 107)) ;; 1 arguments
(global $svc_peek_long i32 (i32.const 108)) ;; 1 arguments
(global $svc_peek_ullong i32 (i32.const 109)) ;; 1 arguments
(global $svc_peek_ulong i32 (i32.const 110)) ;; 1 arguments
(global $svc_peek_size_t i32 (i32.const 111)) ;; 1 arguments
(global $svc_poke_llong i32 (i32.const 112)) ;; 2 arguments
(global $svc_poke_long i32 (i32.const 113)) ;; 2 arguments
(global $svc_poke_ullong i32 (i32.const 114)) ;; 2 arguments
(global $svc_poke_ulong i32 (i32.const 115)) ;; 2 arguments
(global $svc_poke_size_t i32 (i32.const 116)) ;; 2 arguments
(global $svc_sizeof_char i32 (i32.const 117)) ;; 0 arguments
(global $svc_sizeof_short i32 (i32.const 118)) ;; 0 arguments
(global $svc_sizeof_int i32 (i32.const 119)) ;; 0 arguments
(global $svc_sizeof_llong i32 (i32.const 120)) ;; 0 arguments
(global $svc_sizeof_long i32 (i32.const 121)) ;; 0 arguments
(global $svc_sizeof_size_t i32 (i32.const 122)) ;; 0 arguments
(global $svc_c_d_name i32 (i32.const 123)) ;; 1 arguments
(global $svc_closedir i32 (i32.const 124)) ;; 1 arguments
(global $svc_opendir i32 (i32.const 125)) ;; 1 arguments
(global $svc_readdir i32 (i32.const 126)) ;; 1 arguments
(global $svc_chdir i32 (i32.const 127)) ;; 1 arguments
(global $svc_mkdir i32 (i32.const 128)) ;; 2 arguments
(global $svc_getcwd i32 (i32.const 129)) ;; 2 arguments
(global $svc_set_permissions i32 (i32.const 130)) ;; 2 arguments
(global $svc_get_permissions i32 (i32.const 131)) ;; 1 arguments
(global $svc_getcpu i32 (i32.const 132)) ;; 2 arguments
(global $svc_want_gmp i32 (i32.const 133)) ;; 0 arguments
(global $svc_want_imath i32 (i32.const 134)) ;; 0 arguments
(global $svc_new_mpz i32 (i32.const 135)) ;; 0 arguments
(global $svc_mpz_abs i32 (i32.const 136)) ;; 2 arguments
(global $svc_mpz_add i32 (i32.const 137)) ;; 3 arguments
(global $svc_mpz_and i32 (i32.const 138)) ;; 3 arguments
(global $svc_mpz_cmp i32 (i32.const 139)) ;; 2 arguments
(global $svc_mpz_get_d i32 (i32.const 140)) ;; 1 arguments
(global $svc_mpz_get_si i32 (i32.const 141)) ;; 1 arguments
(global $svc_mpz_init_set_si i32 (i32.const 142)) ;; 2 arguments
(global $svc_mpz_init_set_ui i32 (i32.const 143)) ;; 2 arguments
(global $svc_mpz_ior i32 (i32.const 144)) ;; 3 arguments
(global $svc_mpz_mul i32 (i32.const 145)) ;; 3 arguments
(global $svc_mpz_mul_2exp i32 (i32.const 146)) ;; 3 arguments
(global $svc_mpz_neg i32 (i32.const 147)) ;; 2 arguments
(global $svc_mpz_popcount i32 (i32.const 148)) ;; 1 arguments
(global $svc_mpz_sub i32 (i32.const 149)) ;; 3 arguments
(global $svc_mpz_fdiv_q_2exp i32 (i32.const 150)) ;; 3 arguments
(global $svc_mpz_tdiv_qr i32 (i32.const 151)) ;; 4 arguments
(global $svc_mpz_tstbit i32 (i32.const 152)) ;; 2 arguments
(global $svc_mpz_xor i32 (i32.const 153)) ;; 3 arguments
(global $svc_mpz_get_f i32 (i32.const 154)) ;; 1 arguments
(global $svc_mpz_init_set_si64 i32 (i32.const 155)) ;; 2 arguments
(global $svc_mpz_init_set_ui64 i32 (i32.const 156)) ;; 2 arguments
(global $svc_mpz_get_si64 i32 (i32.const 157)) ;; 1 arguments
(global $svc_mpz_log2 i32 (i32.const 158)) ;; 1 arguments
(global $svc_gettimeofday i32 (i32.const 159)) ;; 2 arguments
(global $svc_E2BIG i32 (i32.const 160)) ;; 0 arguments
(global $svc_EACCES i32 (i32.const 161)) ;; 0 arguments
(global $svc_EADDRINUSE i32 (i32.const 162)) ;; 0 arguments
(global $svc_EADDRNOTAVAIL i32 (i32.const 163)) ;; 0 arguments
(global $svc_EADV i32 (i32.const 164)) ;; 0 arguments
(global $svc_EAFNOSUPPORT i32 (i32.const 165)) ;; 0 arguments
(global $svc_EAGAIN i32 (i32.const 166)) ;; 0 arguments
(global $svc_EALREADY i32 (i32.const 167)) ;; 0 arguments
(global $svc_EBADF i32 (i32.const 168)) ;; 0 arguments
(global $svc_EBADMSG i32 (i32.const 169)) ;; 0 arguments
(global $svc_EBADRPC i32 (i32.const 170)) ;; 0 arguments
(global $svc_EBUSY i32 (i32.const 171)) ;; 0 arguments
(global $svc_ECHILD i32 (i32.const 172)) ;; 0 arguments
(global $svc_ECOMM i32 (i32.const 173)) ;; 0 arguments
(global $svc_ECONNABORTED i32 (i32.const 174)) ;; 0 arguments
(global $svc_ECONNREFUSED i32 (i32.const 175)) ;; 0 arguments
(global $svc_ECONNRESET i32 (i32.const 176)) ;; 0 arguments
(global $svc_EDEADLK i32 (i32.const 177)) ;; 0 arguments
(global $svc_EDESTADDRREQ i32 (i32.const 178)) ;; 0 arguments
(global $svc_EDIRTY i32 (i32.const 179)) ;; 0 arguments
(global $svc_EDOM i32 (i32.const 180)) ;; 0 arguments
(global $svc_EDQUOT i32 (i32.const 181)) ;; 0 arguments
(global $svc_EEXIST i32 (i32.const 182)) ;; 0 arguments
(global $svc_EFAULT i32 (i32.const 183)) ;; 0 arguments
(global $svc_EFBIG i32 (i32.const 184)) ;; 0 arguments
(global $svc_EFTYPE i32 (i32.const 185)) ;; 0 arguments
(global $svc_EHOSTDOWN i32 (i32.const 186)) ;; 0 arguments
(global $svc_EHOSTUNREACH i32 (i32.const 187)) ;; 0 arguments
(global $svc_EIDRM i32 (i32.const 188)) ;; 0 arguments
(global $svc_EILSEQ i32 (i32.const 189)) ;; 0 arguments
(global $svc_EINPROGRESS i32 (i32.const 190)) ;; 0 arguments
(global $svc_EINTR i32 (i32.const 191)) ;; 0 arguments
(global $svc_EINVAL i32 (i32.const 192)) ;; 0 arguments
(global $svc_EIO i32 (i32.const 193)) ;; 0 arguments
(global $svc_EISCONN i32 (i32.const 194)) ;; 0 arguments
(global $svc_EISDIR i32 (i32.const 195)) ;; 0 arguments
(global $svc_ELOOP i32 (i32.const 196)) ;; 0 arguments
(global $svc_EMFILE i32 (i32.const 197)) ;; 0 arguments
(global $svc_EMLINK i32 (i32.const 198)) ;; 0 arguments
(global $svc_EMSGSIZE i32 (i32.const 199)) ;; 0 arguments
(global $svc_EMULTIHOP i32 (i32.const 200)) ;; 0 arguments
(global $svc_ENAMETOOLONG i32 (i32.const 201)) ;; 0 arguments
(global $svc_ENETDOWN i32 (i32.const 202)) ;; 0 arguments
(global $svc_ENETRESET i32 (i32.const 203)) ;; 0 arguments
(global $svc_ENETUNREACH i32 (i32.const 204)) ;; 0 arguments
(global $svc_ENFILE i32 (i32.const 205)) ;; 0 arguments
(global $svc_ENOBUFS i32 (i32.const 206)) ;; 0 arguments
(global $svc_ENODATA i32 (i32.const 207)) ;; 0 arguments
(global $svc_ENODEV i32 (i32.const 208)) ;; 0 arguments
(global $svc_ENOENT i32 (i32.const 209)) ;; 0 arguments
(global $svc_ENOEXEC i32 (i32.const 210)) ;; 0 arguments
(global $svc_ENOLCK i32 (i32.const 211)) ;; 0 arguments
(global $svc_ENOLINK i32 (i32.const 212)) ;; 0 arguments
(global $svc_ENOMEM i32 (i32.const 213)) ;; 0 arguments
(global $svc_ENOMSG i32 (i32.const 214)) ;; 0 arguments
(global $svc_ENONET i32 (i32.const 215)) ;; 0 arguments
(global $svc_ENOPROTOOPT i32 (i32.const 216)) ;; 0 arguments
(global $svc_ENOSPC i32 (i32.const 217)) ;; 0 arguments
(global $svc_ENOSR i32 (i32.const 218)) ;; 0 arguments
(global $svc_ENOSTR i32 (i32.const 219)) ;; 0 arguments
(global $svc_ENOSYS i32 (i32.const 220)) ;; 0 arguments
(global $svc_ENOTBLK i32 (i32.const 221)) ;; 0 arguments
(global $svc_ENOTCONN i32 (i32.const 222)) ;; 0 arguments
(global $svc_ENOTDIR i32 (i32.const 223)) ;; 0 arguments
(global $svc_ENOTEMPTY i32 (i32.const 224)) ;; 0 arguments
(global $svc_ENOTSOCK i32 (i32.const 225)) ;; 0 arguments
(global $svc_ENOTSUP i32 (i32.const 226)) ;; 0 arguments
(global $svc_ENOTTY i32 (i32.const 227)) ;; 0 arguments
(global $svc_ENXIO i32 (i32.const 228)) ;; 0 arguments
(global $svc_EOPNOTSUPP i32 (i32.const 229)) ;; 0 arguments
(global $svc_EPERM i32 (i32.const 230)) ;; 0 arguments
(global $svc_EPFNOSUPPORT i32 (i32.const 231)) ;; 0 arguments
(global $svc_EPIPE i32 (i32.const 232)) ;; 0 arguments
(global $svc_EPROCLIM i32 (i32.const 233)) ;; 0 arguments
(global $svc_EPROCUNAVAIL i32 (i32.const 234)) ;; 0 arguments
(global $svc_EPROGMISMATCH i32 (i32.const 235)) ;; 0 arguments
(global $svc_EPROGUNAVAIL i32 (i32.const 236)) ;; 0 arguments
(global $svc_EPROTO i32 (i32.const 237)) ;; 0 arguments
(global $svc_EPROTONOSUPPORT i32 (i32.const 238)) ;; 0 arguments
(global $svc_EPROTOTYPE i32 (i32.const 239)) ;; 0 arguments
(global $svc_ERANGE i32 (i32.const 240)) ;; 0 arguments
(global $svc_EREMCHG i32 (i32.const 241)) ;; 0 arguments
(global $svc_EREMOTE i32 (i32.const 242)) ;; 0 arguments
(global $svc_EROFS i32 (i32.const 243)) ;; 0 arguments
(global $svc_ERPCMISMATCH i32 (i32.const 244)) ;; 0 arguments
(global $svc_ERREMOTE i32 (i32.const 245)) ;; 0 arguments
(global $svc_ESHUTDOWN i32 (i32.const 246)) ;; 0 arguments
(global $svc_ESOCKTNOSUPPORT i32 (i32.const 247)) ;; 0 arguments
(global $svc_ESPIPE i32 (i32.const 248)) ;; 0 arguments
(global $svc_ESRCH i32 (i32.const 249)) ;; 0 arguments
(global $svc_ESRMNT i32 (i32.const 250)) ;; 0 arguments
(global $svc_ESTALE i32 (i32.const 251)) ;; 0 arguments
(global $svc_ETIME i32 (i32.const 252)) ;; 0 arguments
(global $svc_ETIMEDOUT i32 (i32.const 253)) ;; 0 arguments
(global $svc_ETOOMANYREFS i32 (i32.const 254)) ;; 0 arguments
(global $svc_ETXTBSY i32 (i32.const 255)) ;; 0 arguments
(global $svc_EUSERS i32 (i32.const 256)) ;; 0 arguments
(global $svc_EWOULDBLOCK i32 (i32.const 257)) ;; 0 arguments
(global $svc_EXDEV i32 (i32.const 258)) ;; 0 arguments
(global $svc_addr_errno i32 (i32.const 259)) ;; 0 arguments
(global $svc_strerror_r i32 (i32.const 260)) ;; 3 arguments
(global $svc_get_executable_path i32 (i32.const 261)) ;; 0 arguments
(global $svc_getenv i32 (i32.const 262)) ;; 1 arguments
(global $svc_environ i32 (i32.const 263)) ;; 0 arguments
(global $svc_unsetenv i32 (i32.const 264)) ;; 1 arguments
(global $svc_setenv i32 (i32.const 265)) ;; 3 arguments
(global $svc_F_SETFL i32 (i32.const 266)) ;; 0 arguments
(global $svc_O_NONBLOCK i32 (i32.const 267)) ;; 0 arguments
(global $svc_SOL_SOCKET i32 (i32.const 268)) ;; 0 arguments
(global $svc_SO_DEBUG i32 (i32.const 269)) ;; 0 arguments
(global $svc_SO_ERROR i32 (i32.const 270)) ;; 0 arguments
(global $svc_SO_REUSEADDR i32 (i32.const 271)) ;; 0 arguments
(global $svc_SO_TYPE i32 (i32.const 272)) ;; 0 arguments
(global $svc_accept i32 (i32.const 273)) ;; 3 arguments
(global $svc_bind i32 (i32.const 274)) ;; 3 arguments
(global $svc_close i32 (i32.const 275)) ;; 1 arguments
(global $svc_connect i32 (i32.const 276)) ;; 3 arguments
(global $svc_fcntl i32 (i32.const 277)) ;; 3 arguments
(global $svc_getsockopt i32 (i32.const 278)) ;; 5 arguments
(global $svc_listen i32 (i32.const 279)) ;; 2 arguments
(global $svc_recv i32 (i32.const 280)) ;; 4 arguments
(global $svc_send i32 (i32.const 281)) ;; 4 arguments
(global $svc_setsockopt i32 (i32.const 282)) ;; 5 arguments
(global $svc_socket i32 (i32.const 283)) ;; 3 arguments

(global $static_names_end i32 (i32.const 0xc074))

(data (i32.const 0x8000)
  "\42\00\4f\00\4b\00\43\27\00\43\00\41\00\53\27\00\50\00\52\00\49\00\53\00\55\00\59\00\42\27\00\5a\00\4a\00\4b\32\00\4b\33\00\4b\34\00\43\27\42\00\4c\00\4b\4b\00\4b\41\00\54\33\00\54\34\00\54\35"
  "\00\54\36\00\54\37\00\54\38\00\54\39\00\54\31\30\00\54\31\31\00\54\31\32\00\54\31\33\00\54\31\34\00\54\31\35\00\54\31\36\00\54\41\47\30\00\54\41\47\31\00\54\41\47\32\00\54\41\47\33\00\54\41\47"
  "\34\00\54\41\47\35\00\54\41\47\36\00\54\41\47\37\00\54\41\47\38\00\54\41\47\39\00\54\41\47\31\30\00\54\41\47\31\31\00\54\41\47\31\32\00\54\41\47\31\33\00\54\41\47\31\34\00\54\41\47\31\35\00\54"
  "\41\47\31\36\00\54\41\47\31\37\00\54\41\47\31\38\00\54\41\47\31\39\00\54\41\47\32\30\00\54\41\47\32\31\00\54\41\47\32\32\00\54\41\47\32\33\00\54\41\47\32\34\00\54\41\47\32\35\00\54\41\47\32\36"
  "\00\54\41\47\32\37\00\54\41\47\32\38\00\54\41\47\32\39\00\54\41\47\33\30\00\54\41\47\33\31\00\54\41\47\33\32\00\2b\00\2d\00\2a\00\71\75\6f\74\00\72\65\6d\00\75\2b\00\75\2d\00\75\2a\00\75\71\75"
  "\6f\74\00\75\72\65\6d\00\73\75\62\74\72\61\63\74\00\75\73\75\62\74\72\61\63\74\00\6e\65\67\00\75\6e\65\67\00\61\6e\64\00\6f\72\00\78\6f\72\00\69\6e\76\00\73\68\6c\00\73\68\72\00\61\73\68\72\00"
  "\70\6f\70\63\6f\75\6e\74\00\63\6c\7a\00\63\74\7a\00\64\2b\00\64\2d\00\64\2a\00\64\2f\00\64\6e\65\67\00\69\74\6f\64\00\49\74\6f\64\00\75\74\6f\64\00\64\74\6f\69\00\64\3d\3d\00\64\2f\3d\00\64\3c"
  "\00\64\3c\3d\00\64\3e\00\64\3e\3d\00\64\74\6f\66\00\66\74\6f\64\00\66\2b\00\66\2d\00\66\2a\00\66\2f\00\66\6e\65\67\00\49\74\6f\66\00\69\74\6f\66\00\75\74\6f\66\00\66\74\6f\69\00\66\3d\3d\00\66"
  "\2f\3d\00\66\3c\00\66\3c\3d\00\66\3e\00\66\3e\3d\00\62\73\2b\2b\00\62\73\2b\2b\2e\00\62\73\3d\3d\00\62\73\2f\3d\00\62\73\3c\00\62\73\3c\3d\00\62\73\3e\00\62\73\3e\3d\00\62\73\63\6d\70\00\62\73"
  "\75\6e\70\61\63\6b\00\62\73\72\65\70\6c\69\63\61\74\65\00\62\73\6c\65\6e\67\74\68\00\62\73\73\75\62\73\74\72\00\62\73\69\6e\64\65\78\00\62\73\6e\65\77\00\62\73\72\65\61\64\00\62\73\77\72\69\74"
  "\65\00\62\73\66\72\65\65\7a\65\00\62\73\61\70\70\62\79\74\65\00\62\73\61\70\70\63\68\61\72\00\6f\72\64\00\63\68\72\00\3d\3d\00\2f\3d\00\3c\00\75\3c\00\75\3c\3d\00\75\3e\00\75\3e\3d\00\3c\3d\00"
  "\3e\00\3e\3d\00\66\70\2b\00\66\70\32\70\00\66\70\6e\65\77\00\66\70\66\69\6e\00\66\70\32\62\73\00\62\73\32\66\70\00\73\65\71\00\69\63\6d\70\00\75\63\6d\70\00\72\6e\66\00\66\72\6f\6d\55\54\46\38"
  "\00\68\65\61\64\55\54\46\38\00\74\61\69\6c\55\54\46\38\00\49\4f\2e\3e\3e\3d\00\49\4f\2e\3e\3e\00\49\4f\2e\72\65\74\75\72\6e\00\49\4f\2e\73\65\72\69\61\6c\69\7a\65\00\49\4f\2e\70\72\69\6e\74\00"
  "\49\4f\2e\64\65\73\65\72\69\61\6c\69\7a\65\00\49\4f\2e\73\74\64\69\6e\00\49\4f\2e\73\74\64\6f\75\74\00\49\4f\2e\73\74\64\65\72\72\00\49\4f\2e\67\65\74\41\72\67\52\65\66\00\49\4f\2e\70\65\72\66"
  "\6f\72\6d\49\4f\00\49\4f\2e\61\74\6f\6d\69\63\00\49\4f\2e\67\63\00\49\4f\2e\73\74\61\74\73\00\49\4f\2e\70\70\00\49\4f\2e\6c\61\7a\79\42\69\6e\64\00\49\4f\2e\73\74\72\69\63\74\00\72\61\69\73\65"
  "\00\63\61\74\63\68\00\63\61\74\63\68\72\00\41\2e\61\6c\6c\6f\63\00\41\2e\63\6f\70\79\00\41\2e\73\69\7a\65\00\41\2e\72\65\61\64\00\41\2e\77\72\69\74\65\00\41\2e\74\72\75\6e\63\00\41\2e\3d\3d\00"
  "\64\79\6e\73\79\6d\00\49\4f\2e\66\6f\72\6b\00\49\4f\2e\74\68\69\64\00\74\68\6e\75\6d\00\49\4f\2e\74\68\72\6f\77\74\6f\00\49\4f\2e\79\69\65\6c\64\00\49\4f\2e\6e\65\77\6d\76\61\72\00\49\4f\2e\74"
  "\61\6b\65\6d\76\61\72\00\49\4f\2e\70\75\74\6d\76\61\72\00\49\4f\2e\72\65\61\64\6d\76\61\72\00\49\4f\2e\74\72\79\74\61\6b\65\6d\76\61\72\00\49\4f\2e\74\72\79\70\75\74\6d\76\61\72\00\49\4f\2e\74"
  "\72\79\72\65\61\64\6d\76\61\72\00\49\4f\2e\74\68\72\65\61\64\64\65\6c\61\79\00\49\4f\2e\74\68\72\65\61\64\73\74\61\74\75\73\00\49\4f\2e\67\65\74\6d\61\73\6b\69\6e\67\73\74\61\74\65\00\49\4f\2e"
  "\73\65\74\6d\61\73\6b\69\6e\67\73\74\61\74\65\00\70\61\63\6b\43\53\74\72\69\6e\67\00\70\61\63\6b\43\53\74\72\69\6e\67\4c\65\6e\00\62\73\67\72\61\62\00\62\73\67\72\61\62\6c\65\6e\00\74\6f\50\74"
  "\72\00\74\6f\49\6e\74\00\74\6f\44\62\6c\00\74\6f\46\6c\74\00\66\72\6f\6d\44\62\6c\00\66\72\6f\6d\46\6c\74\00\74\6f\46\75\6e\50\74\72\00\49\4f\2e\63\63\61\6c\6c\00\69\73\69\6e\74\00\53\50\6e\65"
  "\77\00\53\50\64\65\72\65\66\00\53\50\66\72\65\65\00\57\6b\6e\65\77\00\57\6b\6e\65\77\66\69\6e\00\57\6b\64\65\72\65\66\00\57\6b\66\69\6e\61\6c\00\62\69\6e\69\6e\74\32\00\62\69\6e\69\6e\74\31\00"
  "\62\69\6e\64\62\6c\32\00\62\69\6e\64\62\6c\31\00\62\69\6e\62\73\32\00\62\69\6e\62\73\31\00\75\6e\69\6e\74\31\00\75\6e\64\62\6c\31\00\49\4f\2e\77\61\69\74\72\64\66\64\00\49\4f\2e\77\61\69\74\77"
  "\72\66\64\00\49\2b\00\49\2d\00\49\2a\00\49\71\75\6f\74\00\49\72\65\6d\00\49\75\2b\00\49\75\2d\00\49\75\2a\00\49\75\71\75\6f\74\00\49\75\72\65\6d\00\49\73\75\62\74\72\61\63\74\00\49\75\73\75\62"
  "\74\72\61\63\74\00\49\6e\65\67\00\49\75\6e\65\67\00\49\61\6e\64\00\49\6f\72\00\49\78\6f\72\00\49\69\6e\76\00\49\73\68\6c\00\49\73\68\72\00\49\61\73\68\72\00\49\70\6f\70\63\6f\75\6e\74\00\49\63"
  "\6c\7a\00\49\63\74\7a\00\49\3d\3d\00\49\2f\3d\00\49\3c\00\49\75\3c\00\49\75\3c\3d\00\49\75\3e\00\49\75\3e\3d\00\49\3c\3d\00\49\3e\00\49\3e\3d\00\49\69\63\6d\70\00\49\75\63\6d\70\00\69\74\6f\49"
  "\00\49\74\6f\69\00\75\74\6f\55\00\55\74\6f\75\00\74\69\63\6b\00\47\45\54\52\41\57\00\47\45\54\54\49\4d\45\4d\49\43\52\4f\00\47\45\54\42\4f\4f\54\54\49\4d\45\4d\49\43\52\4f\00\61\63\6f\73\00\61"
  "\73\69\6e\00\61\74\61\6e\00\61\74\61\6e\32\00\63\6f\73\00\65\78\70\00\6c\6f\67\00\73\69\6e\00\73\71\72\74\00\74\61\6e\00\73\63\61\6c\62\6e\00\70\6f\77\00\70\6f\6b\65\5f\66\6c\74\36\34\00\70\65"
  "\65\6b\5f\66\6c\74\36\34\00\61\63\6f\73\66\00\61\73\69\6e\66\00\61\74\61\6e\66\00\61\74\61\6e\32\66\00\63\6f\73\66\00\65\78\70\66\00\6c\6f\67\66\00\73\69\6e\66\00\73\71\72\74\66\00\74\61\6e\66"
  "\00\73\63\61\6c\62\6e\66\00\70\6f\77\66\00\70\6f\6b\65\5f\66\6c\74\33\32\00\70\65\65\6b\5f\66\6c\74\33\32\00\6a\73\5f\64\65\62\75\67\00\6a\73\5f\65\76\61\6c\5f\72\75\6e\00\6a\73\5f\65\76\61\6c"
  "\5f\63\61\6c\6c\00\6a\73\5f\73\65\74\5f\68\61\73\6b\65\6c\6c\43\61\6c\6c\62\61\63\6b\00\61\64\64\5f\46\49\4c\45\00\70\75\74\63\68\61\72\00\66\6f\70\65\6e\00\74\6d\70\6e\61\6d\65\00\72\65\6d\6f"
  "\76\65\00\73\79\73\74\65\6d\00\61\64\64\5f\66\64\00\6f\70\65\6e\00\61\64\64\5f\62\75\66\00\61\64\64\5f\63\72\6c\66\00\61\64\64\5f\75\74\66\38\00\61\64\64\5f\62\61\73\65\36\34\5f\65\6e\63\6f\64"
  "\65\72\00\61\64\64\5f\62\61\73\65\36\34\5f\64\65\63\6f\64\65\72\00\63\6c\6f\73\65\62\00\26\63\6c\6f\73\65\62\00\66\6c\75\73\68\62\00\67\65\74\62\00\70\75\74\62\00\75\6e\67\65\74\62\00\6f\70\65"
  "\6e\62\5f\77\72\5f\6d\65\6d\00\6f\70\65\6e\62\5f\72\64\5f\6d\65\6d\00\67\65\74\5f\6d\65\6d\00\72\65\61\64\62\00\77\72\69\74\65\62\00\6d\64\35\41\72\72\61\79\00\6d\64\35\42\46\49\4c\45\00\6d\64"
  "\35\53\74\72\69\6e\67\00\61\64\64\5f\6c\7a\37\37\5f\63\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\6c\7a\37\37\5f\64\65\63\6f\6d\70\72\65\73\73\6f\72\00\6c\7a\37\37\63\00\61\64\64\5f\6c\7a\6d\61"
  "\5f\63\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\6c\7a\6d\61\5f\64\65\63\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\72\6c\65\5f\63\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\72\6c\65\5f\64\65\63"
  "\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\62\77\74\5f\63\6f\6d\70\72\65\73\73\6f\72\00\61\64\64\5f\62\77\74\5f\64\65\63\6f\6d\70\72\65\73\73\6f\72\00\63\61\6c\6c\6f\63\00\72\65\61\6c\6c\6f\63"
  "\00\66\72\65\65\00\26\66\72\65\65\00\69\73\77\69\6e\64\6f\77\73\00\69\73\6d\61\63\6f\73\00\69\73\6c\69\6e\75\78\00\6d\61\6c\6c\6f\63\00\6d\65\6d\63\70\79\00\6d\65\6d\6d\6f\76\65\00\73\74\72\6c"
  "\65\6e\00\73\74\72\63\70\79\00\70\65\65\6b\50\74\72\00\70\65\65\6b\57\6f\72\64\00\70\6f\6b\65\50\74\72\00\70\6f\6b\65\57\6f\72\64\00\70\65\65\6b\5f\75\69\6e\74\38\00\70\6f\6b\65\5f\75\69\6e\74"
  "\38\00\70\65\65\6b\5f\75\69\6e\74\31\36\00\70\6f\6b\65\5f\75\69\6e\74\31\36\00\70\65\65\6b\5f\75\69\6e\74\33\32\00\70\6f\6b\65\5f\75\69\6e\74\33\32\00\70\65\65\6b\5f\75\69\6e\74\36\34\00\70\6f"
  "\6b\65\5f\75\69\6e\74\36\34\00\70\65\65\6b\5f\75\69\6e\74\00\70\6f\6b\65\5f\75\69\6e\74\00\70\65\65\6b\5f\69\6e\74\38\00\70\6f\6b\65\5f\69\6e\74\38\00\70\65\65\6b\5f\69\6e\74\31\36\00\70\6f\6b"
  "\65\5f\69\6e\74\31\36\00\70\65\65\6b\5f\69\6e\74\33\32\00\70\6f\6b\65\5f\69\6e\74\33\32\00\70\65\65\6b\5f\69\6e\74\36\34\00\70\6f\6b\65\5f\69\6e\74\36\34\00\70\65\65\6b\5f\69\6e\74\00\70\6f\6b"
  "\65\5f\69\6e\74\00\70\65\65\6b\5f\6c\6c\6f\6e\67\00\70\65\65\6b\5f\6c\6f\6e\67\00\70\65\65\6b\5f\75\6c\6c\6f\6e\67\00\70\65\65\6b\5f\75\6c\6f\6e\67\00\70\65\65\6b\5f\73\69\7a\65\5f\74\00\70\6f"
  "\6b\65\5f\6c\6c\6f\6e\67\00\70\6f\6b\65\5f\6c\6f\6e\67\00\70\6f\6b\65\5f\75\6c\6c\6f\6e\67\00\70\6f\6b\65\5f\75\6c\6f\6e\67\00\70\6f\6b\65\5f\73\69\7a\65\5f\74\00\73\69\7a\65\6f\66\5f\63\68\61"
  "\72\00\73\69\7a\65\6f\66\5f\73\68\6f\72\74\00\73\69\7a\65\6f\66\5f\69\6e\74\00\73\69\7a\65\6f\66\5f\6c\6c\6f\6e\67\00\73\69\7a\65\6f\66\5f\6c\6f\6e\67\00\73\69\7a\65\6f\66\5f\73\69\7a\65\5f\74"
  "\00\63\5f\64\5f\6e\61\6d\65\00\63\6c\6f\73\65\64\69\72\00\6f\70\65\6e\64\69\72\00\72\65\61\64\64\69\72\00\63\68\64\69\72\00\6d\6b\64\69\72\00\67\65\74\63\77\64\00\73\65\74\5f\70\65\72\6d\69\73"
  "\73\69\6f\6e\73\00\67\65\74\5f\70\65\72\6d\69\73\73\69\6f\6e\73\00\67\65\74\63\70\75\00\77\61\6e\74\5f\67\6d\70\00\77\61\6e\74\5f\69\6d\61\74\68\00\6e\65\77\5f\6d\70\7a\00\6d\70\7a\5f\61\62\73"
  "\00\6d\70\7a\5f\61\64\64\00\6d\70\7a\5f\61\6e\64\00\6d\70\7a\5f\63\6d\70\00\6d\70\7a\5f\67\65\74\5f\64\00\6d\70\7a\5f\67\65\74\5f\73\69\00\6d\70\7a\5f\69\6e\69\74\5f\73\65\74\5f\73\69\00\6d\70"
  "\7a\5f\69\6e\69\74\5f\73\65\74\5f\75\69\00\6d\70\7a\5f\69\6f\72\00\6d\70\7a\5f\6d\75\6c\00\6d\70\7a\5f\6d\75\6c\5f\32\65\78\70\00\6d\70\7a\5f\6e\65\67\00\6d\70\7a\5f\70\6f\70\63\6f\75\6e\74\00"
  "\6d\70\7a\5f\73\75\62\00\6d\70\7a\5f\66\64\69\76\5f\71\5f\32\65\78\70\00\6d\70\7a\5f\74\64\69\76\5f\71\72\00\6d\70\7a\5f\74\73\74\62\69\74\00\6d\70\7a\5f\78\6f\72\00\6d\70\7a\5f\67\65\74\5f\66"
  "\00\6d\70\7a\5f\69\6e\69\74\5f\73\65\74\5f\73\69\36\34\00\6d\70\7a\5f\69\6e\69\74\5f\73\65\74\5f\75\69\36\34\00\6d\70\7a\5f\67\65\74\5f\73\69\36\34\00\6d\70\7a\5f\6c\6f\67\32\00\67\65\74\74\69"
  "\6d\65\6f\66\64\61\79\00\45\32\42\49\47\00\45\41\43\43\45\53\00\45\41\44\44\52\49\4e\55\53\45\00\45\41\44\44\52\4e\4f\54\41\56\41\49\4c\00\45\41\44\56\00\45\41\46\4e\4f\53\55\50\50\4f\52\54\00"
  "\45\41\47\41\49\4e\00\45\41\4c\52\45\41\44\59\00\45\42\41\44\46\00\45\42\41\44\4d\53\47\00\45\42\41\44\52\50\43\00\45\42\55\53\59\00\45\43\48\49\4c\44\00\45\43\4f\4d\4d\00\45\43\4f\4e\4e\41\42"
  "\4f\52\54\45\44\00\45\43\4f\4e\4e\52\45\46\55\53\45\44\00\45\43\4f\4e\4e\52\45\53\45\54\00\45\44\45\41\44\4c\4b\00\45\44\45\53\54\41\44\44\52\52\45\51\00\45\44\49\52\54\59\00\45\44\4f\4d\00\45"
  "\44\51\55\4f\54\00\45\45\58\49\53\54\00\45\46\41\55\4c\54\00\45\46\42\49\47\00\45\46\54\59\50\45\00\45\48\4f\53\54\44\4f\57\4e\00\45\48\4f\53\54\55\4e\52\45\41\43\48\00\45\49\44\52\4d\00\45\49"
  "\4c\53\45\51\00\45\49\4e\50\52\4f\47\52\45\53\53\00\45\49\4e\54\52\00\45\49\4e\56\41\4c\00\45\49\4f\00\45\49\53\43\4f\4e\4e\00\45\49\53\44\49\52\00\45\4c\4f\4f\50\00\45\4d\46\49\4c\45\00\45\4d"
  "\4c\49\4e\4b\00\45\4d\53\47\53\49\5a\45\00\45\4d\55\4c\54\49\48\4f\50\00\45\4e\41\4d\45\54\4f\4f\4c\4f\4e\47\00\45\4e\45\54\44\4f\57\4e\00\45\4e\45\54\52\45\53\45\54\00\45\4e\45\54\55\4e\52\45"
  "\41\43\48\00\45\4e\46\49\4c\45\00\45\4e\4f\42\55\46\53\00\45\4e\4f\44\41\54\41\00\45\4e\4f\44\45\56\00\45\4e\4f\45\4e\54\00\45\4e\4f\45\58\45\43\00\45\4e\4f\4c\43\4b\00\45\4e\4f\4c\49\4e\4b\00"
  "\45\4e\4f\4d\45\4d\00\45\4e\4f\4d\53\47\00\45\4e\4f\4e\45\54\00\45\4e\4f\50\52\4f\54\4f\4f\50\54\00\45\4e\4f\53\50\43\00\45\4e\4f\53\52\00\45\4e\4f\53\54\52\00\45\4e\4f\53\59\53\00\45\4e\4f\54"
  "\42\4c\4b\00\45\4e\4f\54\43\4f\4e\4e\00\45\4e\4f\54\44\49\52\00\45\4e\4f\54\45\4d\50\54\59\00\45\4e\4f\54\53\4f\43\4b\00\45\4e\4f\54\53\55\50\00\45\4e\4f\54\54\59\00\45\4e\58\49\4f\00\45\4f\50"
  "\4e\4f\54\53\55\50\50\00\45\50\45\52\4d\00\45\50\46\4e\4f\53\55\50\50\4f\52\54\00\45\50\49\50\45\00\45\50\52\4f\43\4c\49\4d\00\45\50\52\4f\43\55\4e\41\56\41\49\4c\00\45\50\52\4f\47\4d\49\53\4d"
  "\41\54\43\48\00\45\50\52\4f\47\55\4e\41\56\41\49\4c\00\45\50\52\4f\54\4f\00\45\50\52\4f\54\4f\4e\4f\53\55\50\50\4f\52\54\00\45\50\52\4f\54\4f\54\59\50\45\00\45\52\41\4e\47\45\00\45\52\45\4d\43"
  "\48\47\00\45\52\45\4d\4f\54\45\00\45\52\4f\46\53\00\45\52\50\43\4d\49\53\4d\41\54\43\48\00\45\52\52\45\4d\4f\54\45\00\45\53\48\55\54\44\4f\57\4e\00\45\53\4f\43\4b\54\4e\4f\53\55\50\50\4f\52\54"
  "\00\45\53\50\49\50\45\00\45\53\52\43\48\00\45\53\52\4d\4e\54\00\45\53\54\41\4c\45\00\45\54\49\4d\45\00\45\54\49\4d\45\44\4f\55\54\00\45\54\4f\4f\4d\41\4e\59\52\45\46\53\00\45\54\58\54\42\53\59"
  "\00\45\55\53\45\52\53\00\45\57\4f\55\4c\44\42\4c\4f\43\4b\00\45\58\44\45\56\00\26\65\72\72\6e\6f\00\73\74\72\65\72\72\6f\72\5f\72\00\67\65\74\5f\65\78\65\63\75\74\61\62\6c\65\5f\70\61\74\68\00"
  "\67\65\74\65\6e\76\00\65\6e\76\69\72\6f\6e\00\75\6e\73\65\74\65\6e\76\00\73\65\74\65\6e\76\00\46\5f\53\45\54\46\4c\00\4f\5f\4e\4f\4e\42\4c\4f\43\4b\00\53\4f\4c\5f\53\4f\43\4b\45\54\00\53\4f\5f"
  "\44\45\42\55\47\00\53\4f\5f\45\52\52\4f\52\00\53\4f\5f\52\45\55\53\45\41\44\44\52\00\53\4f\5f\54\59\50\45\00\61\63\63\65\70\74\00\62\69\6e\64\00\63\6c\6f\73\65\00\63\6f\6e\6e\65\63\74\00\66\63"
  "\6e\74\6c\00\67\65\74\73\6f\63\6b\6f\70\74\00\6c\69\73\74\65\6e\00\72\65\63\76\00\73\65\6e\64\00\73\65\74\73\6f\63\6b\6f\70\74\00\73\6f\63\6b\65\74\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\16\80\00\00\04\80\00\00\14\80\00\00\00\80\00\00"
  "\09\80\00\00\0b\80\00\00\1a\80\00\00\0d\80\00\00\1c\80\00\00\06\80\00\00\10\80\00\00\12\80\00\00\02\80\00\00\18\80\00\00\1f\80\00\00\21\80\00\00\23\80\00\00\26\80\00\00\29\80\00\00\2c\80\00\00"
  "\30\80\00\00\32\80\00\00\35\80\00\00\38\80\00\00\3b\80\00\00\3e\80\00\00\41\80\00\00\44\80\00\00\47\80\00\00\4a\80\00\00\4d\80\00\00\51\80\00\00\55\80\00\00\59\80\00\00\5d\80\00\00\61\80\00\00"
  "\65\80\00\00\69\80\00\00\6e\80\00\00\73\80\00\00\78\80\00\00\7d\80\00\00\82\80\00\00\87\80\00\00\8c\80\00\00\91\80\00\00\96\80\00\00\9b\80\00\00\a1\80\00\00\a7\80\00\00\ad\80\00\00\b3\80\00\00"
  "\b9\80\00\00\bf\80\00\00\c5\80\00\00\cb\80\00\00\d1\80\00\00\d7\80\00\00\dd\80\00\00\e3\80\00\00\e9\80\00\00\ef\80\00\00\f5\80\00\00\fb\80\00\00\01\81\00\00\07\81\00\00\0d\81\00\00\13\81\00\00"
  "\19\81\00\00\1f\81\00\00\25\81\00\00\27\81\00\00\29\81\00\00\2b\81\00\00\30\81\00\00\48\81\00\00\5b\81\00\00\34\81\00\00\37\81\00\00\3a\81\00\00\3d\81\00\00\43\81\00\00\51\81\00\00\5f\81\00\00"
  "\64\81\00\00\68\81\00\00\6b\81\00\00\6f\81\00\00\73\81\00\00\77\81\00\00\7b\81\00\00\80\81\00\00\89\81\00\00\8d\81\00\00\a7\82\00\00\aa\82\00\00\ad\82\00\00\bd\82\00\00\c0\82\00\00\c2\82\00\00"
  "\af\82\00\00\b2\82\00\00\b6\82\00\00\b9\82\00\00\ea\82\00\00\ef\82\00\00\c4\85\00\00\c7\85\00\00\ca\85\00\00\cd\85\00\00\d3\85\00\00\f1\85\00\00\06\86\00\00\d8\85\00\00\dc\85\00\00\e0\85\00\00"
  "\e4\85\00\00\eb\85\00\00\fb\85\00\00\0b\86\00\00\11\86\00\00\16\86\00\00\1a\86\00\00\1f\86\00\00\24\86\00\00\29\86\00\00\2e\86\00\00\34\86\00\00\3e\86\00\00\43\86\00\00\48\86\00\00\4c\86\00\00"
  "\50\86\00\00\65\86\00\00\69\86\00\00\6c\86\00\00\53\86\00\00\57\86\00\00\5c\86\00\00\60\86\00\00\70\86\00\00\76\86\00\00\7c\86\00\00\81\86\00\00\86\86\00\00\8b\86\00\00\c5\82\00\00\c9\82\00\00"
  "\ce\82\00\00\d4\82\00\00\da\82\00\00\e0\82\00\00\fc\84\00\00\02\85\00\00\08\85\00\00\0e\85\00\00\24\85\00\00\14\85\00\00\1c\85\00\00\70\85\00\00\78\85\00\00\9e\85\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\80\85\00\00\88\85\00\00\a5\85\00\00\90\85\00\00\97\85\00\00\36\85\00\00\d6\81\00\00\d9\81\00\00\dc\81\00\00\df\81\00\00\e2\81\00\00\ec\81\00\00"
  "\e7\81\00\00\f6\81\00\00\f1\81\00\00\fb\81\00\00\ff\81\00\00\03\82\00\00\06\82\00\00\0a\82\00\00\0d\82\00\00\91\81\00\00\94\81\00\00\97\81\00\00\9a\81\00\00\9d\81\00\00\a2\81\00\00\a7\81\00\00"
  "\b1\81\00\00\ac\81\00\00\b6\81\00\00\ba\81\00\00\be\81\00\00\c1\81\00\00\c5\81\00\00\c8\81\00\00\d1\81\00\00\cc\81\00\00\ce\83\00\00\d6\83\00\00\dd\83\00\00\e4\83\00\00\eb\83\00\00\f3\83\00\00"
  "\fb\83\00\00\bb\83\00\00\e6\82\00\00\f4\82\00\00\90\86\00\00\13\83\00\00\1a\83\00\00\20\83\00\00\2a\83\00\00\40\83\00\00\6c\83\00\00\79\83\00\00\86\83\00\00\37\83\00\00\c1\83\00\00\c7\83\00\00"
  "\2d\85\00\00\90\83\00\00\96\83\00\00\a5\83\00\00\b1\83\00\00\00\84\00\00\07\84\00\00\0f\84\00\00\17\84\00\00\1d\84\00\00\28\84\00\00\31\84\00\00\3c\84\00\00\48\84\00\00\53\84\00\00\5f\84\00\00"
  "\6e\84\00\00\7c\84\00\00\8b\84\00\00\9a\84\00\00\aa\84\00\00\bd\84\00\00\d0\84\00\00\dc\84\00\00\11\82\00\00\1c\82\00\00\21\82\00\00\26\82\00\00\2a\82\00\00\2f\82\00\00\33\82\00\00\38\82\00\00"
  "\3e\82\00\00\47\82\00\00\53\82\00\00\5c\82\00\00\65\82\00\00\6d\82\00\00\73\82\00\00\7a\82\00\00\82\82\00\00\8b\82\00\00\95\82\00\00\f8\82\00\00\01\83\00\00\0a\83\00\00\16\82\00\00\eb\84\00\00"
  "\f2\84\00\00\3c\85\00\00\42\85\00\00\4a\85\00\00\57\85\00\00\51\85\00\00\60\85\00\00\68\85\00\00\9f\83\00\00\4f\83\00\00\58\83\00\00\62\83\00\00\ac\85\00\00\b8\85\00\00\00\00\00\00\00\80\00\00"
  "\01\00\00\00\12\00\00\00\02\80\00\00\01\00\00\00\1b\00\00\00\04\80\00\00\01\00\00\00\10\00\00\00\06\80\00\00\02\00\00\00\18\00\00\00\09\80\00\00\01\00\00\00\13\00\00\00\0b\80\00\00\01\00\00\00"
  "\14\00\00\00\0d\80\00\00\02\00\00\00\16\00\00\00\10\80\00\00\01\00\00\00\19\00\00\00\12\80\00\00\01\00\00\00\1a\00\00\00\14\80\00\00\01\00\00\00\11\00\00\00\16\80\00\00\01\00\00\00\0f\00\00\00"
  "\18\80\00\00\01\00\00\00\1c\00\00\00\1a\80\00\00\01\00\00\00\15\00\00\00\1c\80\00\00\02\00\00\00\17\00\00\00\1f\80\00\00\01\00\00\00\1d\00\00\00\21\80\00\00\01\00\00\00\1e\00\00\00\23\80\00\00"
  "\02\00\00\00\1f\00\00\00\26\80\00\00\02\00\00\00\20\00\00\00\29\80\00\00\02\00\00\00\21\00\00\00\2c\80\00\00\03\00\00\00\22\00\00\00\30\80\00\00\01\00\00\00\23\00\00\00\32\80\00\00\02\00\00\00"
  "\24\00\00\00\35\80\00\00\02\00\00\00\25\00\00\00\38\80\00\00\02\00\00\00\26\00\00\00\3b\80\00\00\02\00\00\00\27\00\00\00\3e\80\00\00\02\00\00\00\28\00\00\00\41\80\00\00\02\00\00\00\29\00\00\00"
  "\44\80\00\00\02\00\00\00\2a\00\00\00\47\80\00\00\02\00\00\00\2b\00\00\00\4a\80\00\00\02\00\00\00\2c\00\00\00\4d\80\00\00\03\00\00\00\2d\00\00\00\51\80\00\00\03\00\00\00\2e\00\00\00\55\80\00\00"
  "\03\00\00\00\2f\00\00\00\59\80\00\00\03\00\00\00\30\00\00\00\5d\80\00\00\03\00\00\00\31\00\00\00\61\80\00\00\03\00\00\00\32\00\00\00\65\80\00\00\03\00\00\00\33\00\00\00\69\80\00\00\04\00\00\00"
  "\34\00\00\00\6e\80\00\00\04\00\00\00\35\00\00\00\73\80\00\00\04\00\00\00\36\00\00\00\78\80\00\00\04\00\00\00\37\00\00\00\7d\80\00\00\04\00\00\00\38\00\00\00\82\80\00\00\04\00\00\00\39\00\00\00"
  "\87\80\00\00\04\00\00\00\3a\00\00\00\8c\80\00\00\04\00\00\00\3b\00\00\00\91\80\00\00\04\00\00\00\3c\00\00\00\96\80\00\00\04\00\00\00\3d\00\00\00\9b\80\00\00\05\00\00\00\3e\00\00\00\a1\80\00\00"
  "\05\00\00\00\3f\00\00\00\a7\80\00\00\05\00\00\00\40\00\00\00\ad\80\00\00\05\00\00\00\41\00\00\00\b3\80\00\00\05\00\00\00\42\00\00\00\b9\80\00\00\05\00\00\00\43\00\00\00\bf\80\00\00\05\00\00\00"
  "\44\00\00\00\c5\80\00\00\05\00\00\00\45\00\00\00\cb\80\00\00\05\00\00\00\46\00\00\00\d1\80\00\00\05\00\00\00\47\00\00\00\d7\80\00\00\05\00\00\00\48\00\00\00\dd\80\00\00\05\00\00\00\49\00\00\00"
  "\e3\80\00\00\05\00\00\00\4a\00\00\00\e9\80\00\00\05\00\00\00\4b\00\00\00\ef\80\00\00\05\00\00\00\4c\00\00\00\f5\80\00\00\05\00\00\00\4d\00\00\00\fb\80\00\00\05\00\00\00\4e\00\00\00\01\81\00\00"
  "\05\00\00\00\4f\00\00\00\07\81\00\00\05\00\00\00\50\00\00\00\0d\81\00\00\05\00\00\00\51\00\00\00\13\81\00\00\05\00\00\00\52\00\00\00\19\81\00\00\05\00\00\00\53\00\00\00\1f\81\00\00\05\00\00\00"
  "\54\00\00\00\25\81\00\00\01\00\00\00\55\00\00\00\27\81\00\00\01\00\00\00\56\00\00\00\29\81\00\00\01\00\00\00\57\00\00\00\2b\81\00\00\04\00\00\00\58\00\00\00\30\81\00\00\03\00\00\00\59\00\00\00"
  "\34\81\00\00\02\00\00\00\5c\00\00\00\37\81\00\00\02\00\00\00\5d\00\00\00\3a\81\00\00\02\00\00\00\5e\00\00\00\3d\81\00\00\05\00\00\00\5f\00\00\00\43\81\00\00\04\00\00\00\60\00\00\00\48\81\00\00"
  "\08\00\00\00\5a\00\00\00\51\81\00\00\09\00\00\00\61\00\00\00\5b\81\00\00\03\00\00\00\5b\00\00\00\5f\81\00\00\04\00\00\00\62\00\00\00\64\81\00\00\03\00\00\00\63\00\00\00\68\81\00\00\02\00\00\00"
  "\64\00\00\00\6b\81\00\00\03\00\00\00\65\00\00\00\6f\81\00\00\03\00\00\00\66\00\00\00\73\81\00\00\03\00\00\00\67\00\00\00\77\81\00\00\03\00\00\00\68\00\00\00\7b\81\00\00\04\00\00\00\69\00\00\00"
  "\80\81\00\00\08\00\00\00\6a\00\00\00\89\81\00\00\03\00\00\00\6b\00\00\00\8d\81\00\00\03\00\00\00\6c\00\00\00\91\81\00\00\02\00\00\00\cc\00\00\00\94\81\00\00\02\00\00\00\cd\00\00\00\97\81\00\00"
  "\02\00\00\00\ce\00\00\00\9a\81\00\00\02\00\00\00\cf\00\00\00\9d\81\00\00\04\00\00\00\d0\00\00\00\a2\81\00\00\04\00\00\00\d1\00\00\00\a7\81\00\00\04\00\00\00\d2\00\00\00\ac\81\00\00\04\00\00\00"
  "\d4\00\00\00\b1\81\00\00\04\00\00\00\d3\00\00\00\b6\81\00\00\03\00\00\00\d5\00\00\00\ba\81\00\00\03\00\00\00\d6\00\00\00\be\81\00\00\02\00\00\00\d7\00\00\00\c1\81\00\00\03\00\00\00\d8\00\00\00"
  "\c5\81\00\00\02\00\00\00\d9\00\00\00\c8\81\00\00\03\00\00\00\da\00\00\00\cc\81\00\00\04\00\00\00\dc\00\00\00\d1\81\00\00\04\00\00\00\db\00\00\00\d6\81\00\00\02\00\00\00\bd\00\00\00\d9\81\00\00"
  "\02\00\00\00\be\00\00\00\dc\81\00\00\02\00\00\00\bf\00\00\00\df\81\00\00\02\00\00\00\c0\00\00\00\e2\81\00\00\04\00\00\00\c1\00\00\00\e7\81\00\00\04\00\00\00\c3\00\00\00\ec\81\00\00\04\00\00\00"
  "\c2\00\00\00\f1\81\00\00\04\00\00\00\c5\00\00\00\f6\81\00\00\04\00\00\00\c4\00\00\00\fb\81\00\00\03\00\00\00\c6\00\00\00\ff\81\00\00\03\00\00\00\c7\00\00\00\03\82\00\00\02\00\00\00\c8\00\00\00"
  "\06\82\00\00\03\00\00\00\c9\00\00\00\0a\82\00\00\02\00\00\00\ca\00\00\00\0d\82\00\00\03\00\00\00\cb\00\00\00\11\82\00\00\04\00\00\00\0b\01\00\00\16\82\00\00\05\00\00\00\21\01\00\00\1c\82\00\00"
  "\04\00\00\00\0c\01\00\00\21\82\00\00\04\00\00\00\0d\01\00\00\26\82\00\00\03\00\00\00\0e\01\00\00\2a\82\00\00\04\00\00\00\0f\01\00\00\2f\82\00\00\03\00\00\00\10\01\00\00\33\82\00\00\04\00\00\00"
  "\11\01\00\00\38\82\00\00\05\00\00\00\12\01\00\00\3e\82\00\00\08\00\00\00\13\01\00\00\47\82\00\00\0b\00\00\00\14\01\00\00\53\82\00\00\08\00\00\00\15\01\00\00\5c\82\00\00\08\00\00\00\16\01\00\00"
  "\65\82\00\00\07\00\00\00\17\01\00\00\6d\82\00\00\05\00\00\00\18\01\00\00\73\82\00\00\06\00\00\00\19\01\00\00\7a\82\00\00\07\00\00\00\1a\01\00\00\82\82\00\00\08\00\00\00\1b\01\00\00\8b\82\00\00"
  "\09\00\00\00\1c\01\00\00\95\82\00\00\09\00\00\00\1d\01\00\00\9f\82\00\00\03\00\00\00\11\00\00\00\a3\82\00\00\03\00\00\00\11\00\00\00\a7\82\00\00\02\00\00\00\6d\00\00\00\aa\82\00\00\02\00\00\00"
  "\6e\00\00\00\ad\82\00\00\01\00\00\00\6f\00\00\00\af\82\00\00\02\00\00\00\73\00\00\00\b2\82\00\00\03\00\00\00\74\00\00\00\b6\82\00\00\02\00\00\00\75\00\00\00\b9\82\00\00\03\00\00\00\76\00\00\00"
  "\bd\82\00\00\02\00\00\00\70\00\00\00\c0\82\00\00\01\00\00\00\71\00\00\00\c2\82\00\00\02\00\00\00\72\00\00\00\c5\82\00\00\03\00\00\00\a1\00\00\00\c9\82\00\00\04\00\00\00\a2\00\00\00\ce\82\00\00"
  "\05\00\00\00\a3\00\00\00\d4\82\00\00\05\00\00\00\a4\00\00\00\da\82\00\00\05\00\00\00\a5\00\00\00\e0\82\00\00\05\00\00\00\a6\00\00\00\e6\82\00\00\03\00\00\00\e5\00\00\00\ea\82\00\00\04\00\00\00"
  "\77\00\00\00\ef\82\00\00\04\00\00\00\78\00\00\00\f4\82\00\00\03\00\00\00\e6\00\00\00\f8\82\00\00\08\00\00\00\1e\01\00\00\01\83\00\00\08\00\00\00\1f\01\00\00\0a\83\00\00\08\00\00\00\20\01\00\00"
  "\13\83\00\00\06\00\00\00\e8\00\00\00\1a\83\00\00\05\00\00\00\e9\00\00\00\20\83\00\00\09\00\00\00\ea\00\00\00\2a\83\00\00\0c\00\00\00\eb\00\00\00\37\83\00\00\08\00\00\00\f0\00\00\00\40\83\00\00"
  "\0e\00\00\00\ec\00\00\00\4f\83\00\00\08\00\00\00\2c\01\00\00\58\83\00\00\09\00\00\00\2d\01\00\00\62\83\00\00\09\00\00\00\2e\01\00\00\6c\83\00\00\0c\00\00\00\ed\00\00\00\79\83\00\00\0c\00\00\00"
  "\ee\00\00\00\86\83\00\00\09\00\00\00\ef\00\00\00\90\83\00\00\05\00\00\00\f4\00\00\00\96\83\00\00\08\00\00\00\f5\00\00\00\9f\83\00\00\05\00\00\00\2b\01\00\00\a5\83\00\00\0b\00\00\00\f6\00\00\00"
  "\b1\83\00\00\09\00\00\00\f7\00\00\00\bb\83\00\00\05\00\00\00\e4\00\00\00\c1\83\00\00\05\00\00\00\f1\00\00\00\c7\83\00\00\06\00\00\00\f2\00\00\00\ce\83\00\00\07\00\00\00\dd\00\00\00\d6\83\00\00"
  "\06\00\00\00\de\00\00\00\dd\83\00\00\06\00\00\00\df\00\00\00\e4\83\00\00\06\00\00\00\e0\00\00\00\eb\83\00\00\07\00\00\00\e1\00\00\00\f3\83\00\00\07\00\00\00\e2\00\00\00\fb\83\00\00\04\00\00\00"
  "\e3\00\00\00\00\84\00\00\06\00\00\00\f8\00\00\00\07\84\00\00\07\00\00\00\f9\00\00\00\0f\84\00\00\07\00\00\00\fa\00\00\00\17\84\00\00\05\00\00\00\fb\00\00\00\1d\84\00\00\0a\00\00\00\fc\00\00\00"
  "\28\84\00\00\08\00\00\00\fd\00\00\00\31\84\00\00\0a\00\00\00\fe\00\00\00\3c\84\00\00\0b\00\00\00\ff\00\00\00\48\84\00\00\0a\00\00\00\00\01\00\00\53\84\00\00\0b\00\00\00\01\01\00\00\5f\84\00\00"
  "\0e\00\00\00\02\01\00\00\6e\84\00\00\0d\00\00\00\03\01\00\00\7c\84\00\00\0e\00\00\00\04\01\00\00\8b\84\00\00\0e\00\00\00\05\01\00\00\9a\84\00\00\0f\00\00\00\06\01\00\00\aa\84\00\00\12\00\00\00"
  "\07\01\00\00\bd\84\00\00\12\00\00\00\08\01\00\00\d0\84\00\00\0b\00\00\00\09\01\00\00\dc\84\00\00\0e\00\00\00\0a\01\00\00\eb\84\00\00\06\00\00\00\22\01\00\00\f2\84\00\00\09\00\00\00\23\01\00\00"
  "\fc\84\00\00\05\00\00\00\a7\00\00\00\02\85\00\00\05\00\00\00\a8\00\00\00\08\85\00\00\05\00\00\00\a9\00\00\00\0e\85\00\00\05\00\00\00\aa\00\00\00\14\85\00\00\07\00\00\00\ac\00\00\00\1c\85\00\00"
  "\07\00\00\00\ad\00\00\00\24\85\00\00\08\00\00\00\ab\00\00\00\2d\85\00\00\08\00\00\00\f3\00\00\00\36\85\00\00\05\00\00\00\bc\00\00\00\3c\85\00\00\05\00\00\00\24\01\00\00\42\85\00\00\07\00\00\00"
  "\25\01\00\00\4a\85\00\00\06\00\00\00\26\01\00\00\51\85\00\00\05\00\00\00\28\01\00\00\57\85\00\00\08\00\00\00\27\01\00\00\60\85\00\00\07\00\00\00\29\01\00\00\68\85\00\00\07\00\00\00\2a\01\00\00"
  "\70\85\00\00\07\00\00\00\ae\00\00\00\78\85\00\00\07\00\00\00\af\00\00\00\80\85\00\00\07\00\00\00\b7\00\00\00\88\85\00\00\07\00\00\00\b8\00\00\00\90\85\00\00\06\00\00\00\ba\00\00\00\97\85\00\00"
  "\06\00\00\00\bb\00\00\00\9e\85\00\00\06\00\00\00\b0\00\00\00\a5\85\00\00\06\00\00\00\b9\00\00\00\ac\85\00\00\0b\00\00\00\2f\01\00\00\b8\85\00\00\0b\00\00\00\30\01\00\00\c4\85\00\00\02\00\00\00"
  "\79\00\00\00\c7\85\00\00\02\00\00\00\7a\00\00\00\ca\85\00\00\02\00\00\00\7b\00\00\00\cd\85\00\00\05\00\00\00\7c\00\00\00\d3\85\00\00\04\00\00\00\7d\00\00\00\d8\85\00\00\03\00\00\00\80\00\00\00"
  "\dc\85\00\00\03\00\00\00\81\00\00\00\e0\85\00\00\03\00\00\00\82\00\00\00\e4\85\00\00\06\00\00\00\83\00\00\00\eb\85\00\00\05\00\00\00\84\00\00\00\f1\85\00\00\09\00\00\00\7e\00\00\00\fb\85\00\00"
  "\0a\00\00\00\85\00\00\00\06\86\00\00\04\00\00\00\7f\00\00\00\0b\86\00\00\05\00\00\00\86\00\00\00\11\86\00\00\04\00\00\00\87\00\00\00\16\86\00\00\03\00\00\00\88\00\00\00\1a\86\00\00\04\00\00\00"
  "\89\00\00\00\1f\86\00\00\04\00\00\00\8a\00\00\00\24\86\00\00\04\00\00\00\8b\00\00\00\29\86\00\00\04\00\00\00\8c\00\00\00\2e\86\00\00\05\00\00\00\8d\00\00\00\34\86\00\00\09\00\00\00\8e\00\00\00"
  "\3e\86\00\00\04\00\00\00\8f\00\00\00\43\86\00\00\04\00\00\00\90\00\00\00\48\86\00\00\03\00\00\00\91\00\00\00\4c\86\00\00\03\00\00\00\92\00\00\00\50\86\00\00\02\00\00\00\93\00\00\00\53\86\00\00"
  "\03\00\00\00\97\00\00\00\57\86\00\00\04\00\00\00\98\00\00\00\5c\86\00\00\03\00\00\00\99\00\00\00\60\86\00\00\04\00\00\00\9a\00\00\00\65\86\00\00\03\00\00\00\94\00\00\00\69\86\00\00\02\00\00\00"
  "\95\00\00\00\6c\86\00\00\03\00\00\00\96\00\00\00\70\86\00\00\05\00\00\00\9b\00\00\00\76\86\00\00\05\00\00\00\9c\00\00\00\7c\86\00\00\04\00\00\00\9d\00\00\00\81\86\00\00\04\00\00\00\9e\00\00\00"
  "\86\86\00\00\04\00\00\00\9f\00\00\00\8b\86\00\00\04\00\00\00\a0\00\00\00\90\86\00\00\04\00\00\00\e7\00\00\00\95\86\00\00\06\00\00\00\00\00\00\00\9c\86\00\00\0c\00\00\00\00\00\00\00\a9\86\00\00"
  "\10\00\00\00\00\00\00\00\ba\86\00\00\04\00\00\00\01\00\00\00\bf\86\00\00\04\00\00\00\01\00\00\00\c4\86\00\00\04\00\00\00\01\00\00\00\c9\86\00\00\05\00\00\00\02\00\00\00\cf\86\00\00\03\00\00\00"
  "\01\00\00\00\d3\86\00\00\03\00\00\00\01\00\00\00\d7\86\00\00\03\00\00\00\01\00\00\00\db\86\00\00\03\00\00\00\01\00\00\00\df\86\00\00\04\00\00\00\01\00\00\00\e4\86\00\00\03\00\00\00\01\00\00\00"
  "\e8\86\00\00\06\00\00\00\02\00\00\00\ef\86\00\00\03\00\00\00\02\00\00\00\f3\86\00\00\0a\00\00\00\02\00\00\00\fe\86\00\00\0a\00\00\00\01\00\00\00\09\87\00\00\05\00\00\00\01\00\00\00\0f\87\00\00"
  "\05\00\00\00\01\00\00\00\15\87\00\00\05\00\00\00\01\00\00\00\1b\87\00\00\06\00\00\00\02\00\00\00\22\87\00\00\04\00\00\00\01\00\00\00\27\87\00\00\04\00\00\00\01\00\00\00\2c\87\00\00\04\00\00\00"
  "\01\00\00\00\31\87\00\00\04\00\00\00\01\00\00\00\36\87\00\00\05\00\00\00\01\00\00\00\3c\87\00\00\04\00\00\00\01\00\00\00\41\87\00\00\07\00\00\00\02\00\00\00\49\87\00\00\04\00\00\00\02\00\00\00"
  "\4e\87\00\00\0a\00\00\00\02\00\00\00\59\87\00\00\0a\00\00\00\01\00\00\00\64\87\00\00\08\00\00\00\01\00\00\00\6d\87\00\00\0b\00\00\00\01\00\00\00\79\87\00\00\0c\00\00\00\01\00\00\00\86\87\00\00"
  "\16\00\00\00\01\00\00\00\9d\87\00\00\08\00\00\00\01\00\00\00\a6\87\00\00\07\00\00\00\01\00\00\00\ae\87\00\00\05\00\00\00\02\00\00\00\b4\87\00\00\07\00\00\00\02\00\00\00\bc\87\00\00\06\00\00\00"
  "\01\00\00\00\c3\87\00\00\06\00\00\00\01\00\00\00\ca\87\00\00\06\00\00\00\01\00\00\00\d1\87\00\00\04\00\00\00\03\00\00\00\d6\87\00\00\07\00\00\00\02\00\00\00\de\87\00\00\08\00\00\00\01\00\00\00"
  "\e7\87\00\00\08\00\00\00\01\00\00\00\f0\87\00\00\12\00\00\00\01\00\00\00\03\88\00\00\12\00\00\00\01\00\00\00\16\88\00\00\06\00\00\00\01\00\00\00\1d\88\00\00\07\00\00\00\00\00\00\00\25\88\00\00"
  "\06\00\00\00\01\00\00\00\2c\88\00\00\04\00\00\00\01\00\00\00\31\88\00\00\04\00\00\00\02\00\00\00\36\88\00\00\06\00\00\00\02\00\00\00\3d\88\00\00\0c\00\00\00\00\00\00\00\4a\88\00\00\0c\00\00\00"
  "\02\00\00\00\57\88\00\00\07\00\00\00\03\00\00\00\5f\88\00\00\05\00\00\00\03\00\00\00\65\88\00\00\06\00\00\00\03\00\00\00\6c\88\00\00\08\00\00\00\03\00\00\00\75\88\00\00\08\00\00\00\02\00\00\00"
  "\7e\88\00\00\09\00\00\00\02\00\00\00\88\88\00\00\13\00\00\00\01\00\00\00\9c\88\00\00\15\00\00\00\01\00\00\00\b2\88\00\00\05\00\00\00\03\00\00\00\b8\88\00\00\13\00\00\00\01\00\00\00\cc\88\00\00"
  "\15\00\00\00\01\00\00\00\e2\88\00\00\12\00\00\00\01\00\00\00\f5\88\00\00\14\00\00\00\01\00\00\00\0a\89\00\00\12\00\00\00\01\00\00\00\1d\89\00\00\14\00\00\00\01\00\00\00\32\89\00\00\06\00\00\00"
  "\02\00\00\00\39\89\00\00\07\00\00\00\02\00\00\00\41\89\00\00\04\00\00\00\01\00\00\00\46\89\00\00\05\00\00\00\00\00\00\00\4c\89\00\00\09\00\00\00\00\00\00\00\56\89\00\00\07\00\00\00\00\00\00\00"
  "\5e\89\00\00\07\00\00\00\00\00\00\00\66\89\00\00\06\00\00\00\01\00\00\00\6d\89\00\00\06\00\00\00\03\00\00\00\74\89\00\00\07\00\00\00\03\00\00\00\7c\89\00\00\06\00\00\00\01\00\00\00\83\89\00\00"
  "\06\00\00\00\02\00\00\00\8a\89\00\00\07\00\00\00\01\00\00\00\92\89\00\00\08\00\00\00\01\00\00\00\9b\89\00\00\07\00\00\00\02\00\00\00\a3\89\00\00\08\00\00\00\02\00\00\00\ac\89\00\00\0a\00\00\00"
  "\01\00\00\00\b7\89\00\00\0a\00\00\00\02\00\00\00\c2\89\00\00\0b\00\00\00\01\00\00\00\ce\89\00\00\0b\00\00\00\02\00\00\00\da\89\00\00\0b\00\00\00\01\00\00\00\e6\89\00\00\0b\00\00\00\02\00\00\00"
  "\f2\89\00\00\0b\00\00\00\01\00\00\00\fe\89\00\00\0b\00\00\00\02\00\00\00\0a\8a\00\00\09\00\00\00\01\00\00\00\14\8a\00\00\09\00\00\00\02\00\00\00\1e\8a\00\00\09\00\00\00\01\00\00\00\28\8a\00\00"
  "\09\00\00\00\02\00\00\00\32\8a\00\00\0a\00\00\00\01\00\00\00\3d\8a\00\00\0a\00\00\00\02\00\00\00\48\8a\00\00\0a\00\00\00\01\00\00\00\53\8a\00\00\0a\00\00\00\02\00\00\00\5e\8a\00\00\0a\00\00\00"
  "\01\00\00\00\69\8a\00\00\0a\00\00\00\02\00\00\00\74\8a\00\00\08\00\00\00\01\00\00\00\7d\8a\00\00\08\00\00\00\02\00\00\00\86\8a\00\00\0a\00\00\00\01\00\00\00\91\8a\00\00\09\00\00\00\01\00\00\00"
  "\9b\8a\00\00\0b\00\00\00\01\00\00\00\a7\8a\00\00\0a\00\00\00\01\00\00\00\b2\8a\00\00\0b\00\00\00\01\00\00\00\be\8a\00\00\0a\00\00\00\02\00\00\00\c9\8a\00\00\09\00\00\00\02\00\00\00\d3\8a\00\00"
  "\0b\00\00\00\02\00\00\00\df\8a\00\00\0a\00\00\00\02\00\00\00\ea\8a\00\00\0b\00\00\00\02\00\00\00\f6\8a\00\00\0b\00\00\00\00\00\00\00\02\8b\00\00\0c\00\00\00\00\00\00\00\0f\8b\00\00\0a\00\00\00"
  "\00\00\00\00\1a\8b\00\00\0c\00\00\00\00\00\00\00\27\8b\00\00\0b\00\00\00\00\00\00\00\33\8b\00\00\0d\00\00\00\00\00\00\00\41\8b\00\00\08\00\00\00\01\00\00\00\4a\8b\00\00\08\00\00\00\01\00\00\00"
  "\53\8b\00\00\07\00\00\00\01\00\00\00\5b\8b\00\00\07\00\00\00\01\00\00\00\63\8b\00\00\05\00\00\00\01\00\00\00\69\8b\00\00\05\00\00\00\02\00\00\00\6f\8b\00\00\06\00\00\00\02\00\00\00\76\8b\00\00"
  "\0f\00\00\00\02\00\00\00\86\8b\00\00\0f\00\00\00\01\00\00\00\96\8b\00\00\06\00\00\00\02\00\00\00\9d\8b\00\00\08\00\00\00\00\00\00\00\a6\8b\00\00\0a\00\00\00\00\00\00\00\b1\8b\00\00\07\00\00\00"
  "\00\00\00\00\b9\8b\00\00\07\00\00\00\02\00\00\00\c1\8b\00\00\07\00\00\00\03\00\00\00\c9\8b\00\00\07\00\00\00\03\00\00\00\d1\8b\00\00\07\00\00\00\02\00\00\00\d9\8b\00\00\09\00\00\00\01\00\00\00"
  "\e3\8b\00\00\0a\00\00\00\01\00\00\00\ee\8b\00\00\0f\00\00\00\02\00\00\00\fe\8b\00\00\0f\00\00\00\02\00\00\00\0e\8c\00\00\07\00\00\00\03\00\00\00\16\8c\00\00\07\00\00\00\03\00\00\00\1e\8c\00\00"
  "\0c\00\00\00\03\00\00\00\2b\8c\00\00\07\00\00\00\02\00\00\00\33\8c\00\00\0c\00\00\00\01\00\00\00\40\8c\00\00\07\00\00\00\03\00\00\00\48\8c\00\00\0f\00\00\00\03\00\00\00\58\8c\00\00\0b\00\00\00"
  "\04\00\00\00\64\8c\00\00\0a\00\00\00\02\00\00\00\6f\8c\00\00\07\00\00\00\03\00\00\00\77\8c\00\00\09\00\00\00\01\00\00\00\81\8c\00\00\11\00\00\00\02\00\00\00\93\8c\00\00\11\00\00\00\02\00\00\00"
  "\a5\8c\00\00\0c\00\00\00\01\00\00\00\b2\8c\00\00\08\00\00\00\01\00\00\00\bb\8c\00\00\0c\00\00\00\02\00\00\00\c8\8c\00\00\05\00\00\00\00\00\00\00\ce\8c\00\00\06\00\00\00\00\00\00\00\d5\8c\00\00"
  "\0a\00\00\00\00\00\00\00\e0\8c\00\00\0d\00\00\00\00\00\00\00\ee\8c\00\00\04\00\00\00\00\00\00\00\f3\8c\00\00\0c\00\00\00\00\00\00\00\00\8d\00\00\06\00\00\00\00\00\00\00\07\8d\00\00\08\00\00\00"
  "\00\00\00\00\10\8d\00\00\05\00\00\00\00\00\00\00\16\8d\00\00\07\00\00\00\00\00\00\00\1e\8d\00\00\07\00\00\00\00\00\00\00\26\8d\00\00\05\00\00\00\00\00\00\00\2c\8d\00\00\06\00\00\00\00\00\00\00"
  "\33\8d\00\00\05\00\00\00\00\00\00\00\39\8d\00\00\0c\00\00\00\00\00\00\00\46\8d\00\00\0c\00\00\00\00\00\00\00\53\8d\00\00\0a\00\00\00\00\00\00\00\5e\8d\00\00\07\00\00\00\00\00\00\00\66\8d\00\00"
  "\0c\00\00\00\00\00\00\00\73\8d\00\00\06\00\00\00\00\00\00\00\7a\8d\00\00\04\00\00\00\00\00\00\00\7f\8d\00\00\06\00\00\00\00\00\00\00\86\8d\00\00\06\00\00\00\00\00\00\00\8d\8d\00\00\06\00\00\00"
  "\00\00\00\00\94\8d\00\00\05\00\00\00\00\00\00\00\9a\8d\00\00\06\00\00\00\00\00\00\00\a1\8d\00\00\09\00\00\00\00\00\00\00\ab\8d\00\00\0c\00\00\00\00\00\00\00\b8\8d\00\00\05\00\00\00\00\00\00\00"
  "\be\8d\00\00\06\00\00\00\00\00\00\00\c5\8d\00\00\0b\00\00\00\00\00\00\00\d1\8d\00\00\05\00\00\00\00\00\00\00\d7\8d\00\00\06\00\00\00\00\00\00\00\de\8d\00\00\03\00\00\00\00\00\00\00\e2\8d\00\00"
  "\07\00\00\00\00\00\00\00\ea\8d\00\00\06\00\00\00\00\00\00\00\f1\8d\00\00\05\00\00\00\00\00\00\00\f7\8d\00\00\06\00\00\00\00\00\00\00\fe\8d\00\00\06\00\00\00\00\00\00\00\05\8e\00\00\08\00\00\00"
  "\00\00\00\00\0e\8e\00\00\09\00\00\00\00\00\00\00\18\8e\00\00\0c\00\00\00\00\00\00\00\25\8e\00\00\08\00\00\00\00\00\00\00\2e\8e\00\00\09\00\00\00\00\00\00\00\38\8e\00\00\0b\00\00\00\00\00\00\00"
  "\44\8e\00\00\06\00\00\00\00\00\00\00\4b\8e\00\00\07\00\00\00\00\00\00\00\53\8e\00\00\07\00\00\00\00\00\00\00\5b\8e\00\00\06\00\00\00\00\00\00\00\62\8e\00\00\06\00\00\00\00\00\00\00\69\8e\00\00"
  "\07\00\00\00\00\00\00\00\71\8e\00\00\06\00\00\00\00\00\00\00\78\8e\00\00\07\00\00\00\00\00\00\00\80\8e\00\00\06\00\00\00\00\00\00\00\87\8e\00\00\06\00\00\00\00\00\00\00\8e\8e\00\00\06\00\00\00"
  "\00\00\00\00\95\8e\00\00\0b\00\00\00\00\00\00\00\a1\8e\00\00\06\00\00\00\00\00\00\00\a8\8e\00\00\05\00\00\00\00\00\00\00\ae\8e\00\00\06\00\00\00\00\00\00\00\b5\8e\00\00\06\00\00\00\00\00\00\00"
  "\bc\8e\00\00\07\00\00\00\00\00\00\00\c4\8e\00\00\08\00\00\00\00\00\00\00\cd\8e\00\00\07\00\00\00\00\00\00\00\d5\8e\00\00\09\00\00\00\00\00\00\00\df\8e\00\00\08\00\00\00\00\00\00\00\e8\8e\00\00"
  "\07\00\00\00\00\00\00\00\f0\8e\00\00\06\00\00\00\00\00\00\00\f7\8e\00\00\05\00\00\00\00\00\00\00\fd\8e\00\00\0a\00\00\00\00\00\00\00\08\8f\00\00\05\00\00\00\00\00\00\00\0e\8f\00\00\0c\00\00\00"
  "\00\00\00\00\1b\8f\00\00\05\00\00\00\00\00\00\00\21\8f\00\00\08\00\00\00\00\00\00\00\2a\8f\00\00\0c\00\00\00\00\00\00\00\37\8f\00\00\0d\00\00\00\00\00\00\00\45\8f\00\00\0c\00\00\00\00\00\00\00"
  "\52\8f\00\00\06\00\00\00\00\00\00\00\59\8f\00\00\0f\00\00\00\00\00\00\00\69\8f\00\00\0a\00\00\00\00\00\00\00\74\8f\00\00\06\00\00\00\00\00\00\00\7b\8f\00\00\07\00\00\00\00\00\00\00\83\8f\00\00"
  "\07\00\00\00\00\00\00\00\8b\8f\00\00\05\00\00\00\00\00\00\00\91\8f\00\00\0c\00\00\00\00\00\00\00\9e\8f\00\00\08\00\00\00\00\00\00\00\a7\8f\00\00\09\00\00\00\00\00\00\00\b1\8f\00\00\0f\00\00\00"
  "\00\00\00\00\c1\8f\00\00\06\00\00\00\00\00\00\00\c8\8f\00\00\05\00\00\00\00\00\00\00\ce\8f\00\00\06\00\00\00\00\00\00\00\d5\8f\00\00\06\00\00\00\00\00\00\00\dc\8f\00\00\05\00\00\00\00\00\00\00"
  "\e2\8f\00\00\09\00\00\00\00\00\00\00\ec\8f\00\00\0c\00\00\00\00\00\00\00\f9\8f\00\00\07\00\00\00\00\00\00\00\01\90\00\00\06\00\00\00\00\00\00\00\08\90\00\00\0b\00\00\00\00\00\00\00\14\90\00\00"
  "\05\00\00\00\00\00\00\00\1a\90\00\00\06\00\00\00\00\00\00\00\21\90\00\00\0a\00\00\00\03\00\00\00\2c\90\00\00\13\00\00\00\00\00\00\00\40\90\00\00\06\00\00\00\01\00\00\00\47\90\00\00\07\00\00\00"
  "\00\00\00\00\4f\90\00\00\08\00\00\00\01\00\00\00\58\90\00\00\06\00\00\00\03\00\00\00\5f\90\00\00\07\00\00\00\00\00\00\00\67\90\00\00\0a\00\00\00\00\00\00\00\72\90\00\00\0a\00\00\00\00\00\00\00"
  "\7d\90\00\00\08\00\00\00\00\00\00\00\86\90\00\00\08\00\00\00\00\00\00\00\8f\90\00\00\0c\00\00\00\00\00\00\00\9c\90\00\00\07\00\00\00\00\00\00\00\a4\90\00\00\06\00\00\00\03\00\00\00\ab\90\00\00"
  "\04\00\00\00\03\00\00\00\b0\90\00\00\05\00\00\00\01\00\00\00\b6\90\00\00\07\00\00\00\03\00\00\00\be\90\00\00\05\00\00\00\03\00\00\00\c4\90\00\00\0a\00\00\00\05\00\00\00\cf\90\00\00\06\00\00\00"
  "\02\00\00\00\d6\90\00\00\04\00\00\00\04\00\00\00\db\90\00\00\04\00\00\00\04\00\00\00\e0\90\00\00\0a\00\00\00\05\00\00\00\eb\90\00\00\06\00\00\00\03\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\03\01\00\00\00\00\00\00\00\00\00\00\0e\00\00\00\4d\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\77\00\00\00\00\00\00\00\09\01\00\00\44\00\00\00\e1\00\00\00\ce\00\00\00"
  "\00\00\00\00\9e\00\00\00\a7\00\00\00\c6\00\00\00\ae\00\00\00\f4\00\00\00\3a\00\00\00\00\00\00\00\61\00\00\00\a2\00\00\00\10\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\9d\00\00\00\30\00\00\00"
  "\c4\00\00\00\0c\01\00\00\00\00\00\00\00\00\00\00\ec\00\00\00\00\00\00\00\06\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\e0\00\00\00\e4\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\1f\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\48\00\00\00\26\00\00\00\83\00\00\00\fb\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00\00\00\00\00\00\00\00\00\d1\00\00\00\00\00\00\00"
  "\37\00\00\00\5a\00\00\00\0f\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\8e\00\00\00\d7\00\00\00\bc\00\00\00\39\00\00\00\00\00\00\00\04\00\00\00\00\00\00\00\08\01\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\19\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\90\00\00\00\92\00\00\00\d9\00\00\00\71\00\00\00\6a\00\00\00\87\00\00\00\ee\00\00\00\e9\00\00\00\00\00\00\00\a9\00\00\00"
  "\00\00\00\00\6f\00\00\00\3e\00\00\00\00\00\00\00\d2\00\00\00\00\00\00\00\00\00\00\00\9a\00\00\00\00\00\00\00\00\00\00\00\11\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\cc\00\00\00\14\00\00\00\5b\00\00\00\a3\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\c3\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\0a\01\00\00"
  "\00\00\00\00\1c\01\00\00\2a\00\00\00\3b\00\00\00\00\00\00\00\5f\00\00\00\c8\00\00\00\03\00\00\00\e2\00\00\00\d4\00\00\00\00\00\00\00\d6\00\00\00\33\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\0d\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\bf\00\00\00\00\00\00\00\88\00\00\00\00\00\00\00\e6\00\00\00\22\00\00\00\00\00\00\00\1d\00\00\00\00\00\00\00"
  "\00\00\00\00\1e\01\00\00\89\00\00\00\27\00\00\00\55\00\00\00\6e\00\00\00\7c\00\00\00\00\00\00\00\15\00\00\00\64\00\00\00\00\00\00\00\16\01\00\00\00\00\00\00\36\00\00\00\42\00\00\00\d3\00\00\00"
  "\00\00\00\00\00\00\00\00\09\00\00\00\00\00\00\00\00\00\00\00\a6\00\00\00\00\00\00\00\38\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\6d\00\00\00\7b\00\00\00\25\00\00\00\b3\00\00\00\1a\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\b8\00\00\00\00\00\00\00\72\00\00\00\00\00\00\00\69\00\00\00\06\00\00\00\8f\00\00\00\ef\00\00\00\8b\00\00\00\81\00\00\00\2e\00\00\00\3f\00\00\00"
  "\95\00\00\00\62\00\00\00\68\00\00\00\a0\00\00\00\b6\00\00\00\c7\00\00\00\96\00\00\00\e7\00\00\00\07\01\00\00\0e\01\00\00\00\00\00\00\df\00\00\00\07\00\00\00\da\00\00\00\4c\00\00\00\f5\00\00\00"
  "\fe\00\00\00\b9\00\00\00\00\00\00\00\00\00\00\00\66\00\00\00\00\00\00\00\00\00\00\00\0b\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\75\00\00\00\2b\00\00\00"
  "\3c\00\00\00\00\00\00\00\9b\00\00\00\00\00\00\00\00\00\00\00\b7\00\00\00\af\00\00\00\fa\00\00\00\97\00\00\00\32\00\00\00\ab\00\00\00\00\01\00\00\00\00\00\00\7f\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\c5\00\00\00\84\00\00\00\00\00\00\00\4b\00\00\00\7e\00\00\00\b1\00\00\00\00\00\00\00\d5\00\00\00\21\00\00\00\ff\00\00\00\1e\00\00\00\14\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\28\00\00\00\78\00\00\00\67\00\00\00\85\00\00\00\00\00\00\00\00\00\00\00\05\01\00\00\13\00\00\00\91\00\00\00\a1\00\00\00\aa\00\00\00\43\00\00\00\dd\00\00\00\6b\00\00\00\00\00\00\00\0b\00\00\00"
  "\bd\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\e8\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\8d\00\00\00\c9\00\00\00\1a\01\00\00\1b\00\00\00\00\00\00\00\00\00\00\00\cb\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\46\00\00\00\6c\00\00\00\58\00\00\00\70\00\00\00\8a\00\00\00\12\00\00\00\f1\00\00\00\49\00\00\00\2f\00\00\00\40\00\00\00\5c\00\00\00\53\00\00\00\de\00\00\00"
  "\01\01\00\00\98\00\00\00\a8\00\00\00\f9\00\00\00\db\00\00\00\4a\00\00\00\04\01\00\00\13\01\00\00\1d\01\00\00\be\00\00\00\0f\00\00\00\4e\00\00\00\e3\00\00\00\18\00\00\00\eb\00\00\00\02\01\00\00"
  "\00\00\00\00\00\00\00\00\74\00\00\00\a4\00\00\00\b2\00\00\00\ba\00\00\00\00\00\00\00\cf\00\00\00\00\00\00\00\00\00\00\00\f6\00\00\00\00\00\00\00\2c\00\00\00\3d\00\00\00\51\00\00\00\e5\00\00\00"
  "\00\00\00\00\0a\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\35\00\00\00\4f\00\00\00\80\00\00\00\17\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\94\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\5e\00\00\00\24\00\00\00\ac\00\00\00\99\00\00\00\c1\00\00\00\00\00\00\00\54\00\00\00\57\00\00\00\29\00\00\00\82\00\00\00\73\00\00\00"
  "\dc\00\00\00\56\00\00\00\01\00\00\00\19\01\00\00\00\00\00\00\10\01\00\00\00\00\00\00\00\00\00\00\86\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\08\00\00\00\00\00\00\00\17\00\00\00\d0\00\00\00"
  "\63\00\00\00\00\00\00\00\ad\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\1c\00\00\00\50\00\00\00\00\00\00\00\00\00\00\00\5d\00\00\00\93\00\00\00\9f\00\00\00"
  "\45\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\11\00\00\00\ea\00\00\00\47\00\00\00\fc\00\00\00\41\00\00\00\52\00\00\00\7a\00\00\00\00\00\00\00\0c\00\00\00\00\00\00\00\9c\00\00\00"
  "\00\00\00\00\d8\00\00\00\31\00\00\00\59\00\00\00\8c\00\00\00\00\00\00\00\00\00\00\00\ed\00\00\00\00\00\00\00\a5\00\00\00\b4\00\00\00\00\00\00\00\ca\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\79\00\00\00\00\00\00\00\20\00\00\00\00\00\00\00\c0\00\00\00\c2\00\00\00\f3\00\00\00\b5\00\00\00\65\00\00\00\2d\00\00\00\f8\00\00\00\fd\00\00\00\18\01\00\00\00\00\00\00\76\00\00\00\00\00\00\00"
  "\16\00\00\00\00\00\00\00\00\00\00\00\34\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\0d\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\b0\00\00\00\bb\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\7d\00\00\00\23\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\f7\00\00\00\1b\01\00\00\cd\00\00\00\00\00\00\00\15\01\00\00\12\01\00\00\00\00\00\00\05\00\00\00"
  "\00\00\00\00\00\00\00\00\f0\00\00\00\f2\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\60\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\03\01\00\00\00\00\00\00\25\00\00\00"
  "\48\00\00\00\db\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\5e\00\00\00\af\00\00\00\00\00\00\00\62\00\00\00\00\00\00\00\20\00\00\00\00\00\00\00\00\00\00\00\45\00\00\00\2d\00\00\00"
  "\00\00\00\00\00\00\00\00\65\00\00\00\89\00\00\00\21\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\3d\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\e8\00\00\00\00\00\00\00\b0\00\00\00\00\00\00\00\00\00\00\00\12\01\00\00\6b\00\00\00\00\00\00\00\00\00\00\00\19\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\57\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\7e\00\00\00\ec\00\00\00\00\00\00\00\00\00\00\00\17\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\13\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\56\00\00\00\cb\00\00\00\3c\00\00\00\7c\00\00\00\08\01\00\00\67\00\00\00\a4\00\00\00\0b\00\00\00\a7\00\00\00\32\00\00\00\da\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\63\00\00\00\97\00\00\00\79\00\00\00\ad\00\00\00\ef\00\00\00\05\01\00\00\e9\00\00\00\00\00\00\00\eb\00\00\00\00\00\00\00\69\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\50\00\00\00\11\00\00\00\99\00\00\00\0a\01\00\00\00\00\00\00\39\00\00\00\00\01\00\00\00\00\00\00\1c\01\00\00\00\00\00\00\8d\00\00\00\34\00\00\00\43\00\00\00\7a\00\00\00\0e\01\00\00"
  "\16\01\00\00\00\00\00\00\00\00\00\00\f3\00\00\00\01\00\00\00\72\00\00\00\70\00\00\00\46\00\00\00\f2\00\00\00\00\00\00\00\00\00\00\00\e7\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\7d\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\d0\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\10\01\00\00\ce\00\00\00\87\00\00\00\8a\00\00\00\fc\00\00\00\9a\00\00\00\00\00\00\00\4d\00\00\00"
  "\75\00\00\00\a3\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\0d\00\00\00\40\00\00\00\3b\00\00\00\e3\00\00\00\08\00\00\00\29\00\00\00\61\00\00\00\de\00\00\00\e0\00\00\00\fb\00\00\00\02\01\00\00"
  "\85\00\00\00\f6\00\00\00\04\01\00\00\05\00\00\00\53\00\00\00\83\00\00\00\c6\00\00\00\cd\00\00\00\3f\00\00\00\c2\00\00\00\27\00\00\00\5c\00\00\00\ab\00\00\00\e5\00\00\00\18\01\00\00\1a\01\00\00"
  "\00\00\00\00\7b\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\b3\00\00\00\00\00\00\00\06\00\00\00\d9\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\aa\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\86\00\00\00\2b\00\00\00\d7\00\00\00\00\00\00\00\8b\00\00\00\93\00\00\00\00\00\00\00\82\00\00\00\00\00\00\00\37\00\00\00\00\00\00\00"
  "\a0\00\00\00\b7\00\00\00\0f\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\1f\00\00\00\4a\00\00\00\6f\00\00\00\d2\00\00\00\0e\00\00\00\5f\00\00\00\00\00\00\00\00\00\00\00\22\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\f7\00\00\00\00\00\00\00\00\00\00\00\6d\00\00\00\00\00\00\00\00\00\00\00\9e\00\00\00\01\01\00\00\00\00\00\00"
  "\00\00\00\00\84\00\00\00\f9\00\00\00\5b\00\00\00\c1\00\00\00\c3\00\00\00\cf\00\00\00\f0\00\00\00\15\01\00\00\00\00\00\00\b6\00\00\00\00\00\00\00\00\00\00\00\c8\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\bc\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\5d\00\00\00\00\00\00\00\00\00\00\00\1a\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\96\00\00\00\11\01\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\ac\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\c0\00\00\00\e1\00\00\00\94\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\04\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\0d\01\00\00\14\01\00\00\00\00\00\00\00\00\00\00\0c\01\00\00\09\00\00\00\00\00\00\00\00\00\00\00\c7\00\00\00\00\00\00\00\ca\00\00\00\44\00\00\00\13\01\00\00\1e\00\00\00\00\00\00\00\ed\00\00\00"
  "\8f\00\00\00\00\00\00\00\f1\00\00\00\2c\00\00\00\24\00\00\00\41\00\00\00\fd\00\00\00\38\00\00\00\92\00\00\00\9d\00\00\00\dd\00\00\00\09\01\00\00\00\00\00\00\bf\00\00\00\00\00\00\00\00\00\00\00"
  "\15\00\00\00\00\00\00\00\49\00\00\00\ee\00\00\00\00\00\00\00\9b\00\00\00\00\00\00\00\76\00\00\00\91\00\00\00\00\00\00\00\4f\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\0a\00\00\00\23\00\00\00"
  "\31\00\00\00\6e\00\00\00\30\00\00\00\a2\00\00\00\00\00\00\00\00\00\00\00\a5\00\00\00\55\00\00\00\a6\00\00\00\b5\00\00\00\f4\00\00\00\07\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\42\00\00\00\35\00\00\00\80\00\00\00\dc\00\00\00\10\00\00\00\4b\00\00\00\78\00\00\00\90\00\00\00\00\00\00\00\00\00\00\00\33\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\e4\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\4c\00\00\00\12\00\00\00\6a\00\00\00\9f\00\00\00\68\00\00\00\0f\01\00\00\b4\00\00\00\0b\01\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\73\00\00\00\00\00\00\00\00\00\00\00\58\00\00\00\fa\00\00\00\51\00\00\00\e6\00\00\00\47\00\00\00\16\00\00\00\f8\00\00\00\bd\00\00\00\00\00\00\00\74\00\00\00"
  "\64\00\00\00\7f\00\00\00\18\00\00\00\b8\00\00\00\52\00\00\00\a1\00\00\00\b9\00\00\00\f5\00\00\00\14\00\00\00\00\00\00\00\95\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\02\00\00\00"
  "\ba\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\1c\00\00\00\5a\00\00\00\e2\00\00\00\6c\00\00\00\d6\00\00\00\00\00\00\00\d1\00\00\00\88\00\00\00\ae\00\00\00\19\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\00\00\00\00\2e\00\00\00\00\00\00\00\54\00\00\00\00\00\00\00\8e\00\00\00\00\00\00\00\36\00\00\00\66\00\00\00\8c\00\00\00\59\00\00\00\a8\00\00\00\00\00\00\00\00\00\00\00"
  "\2a\00\00\00\ff\00\00\00\26\00\00\00\00\00\00\00\00\00\00\00\1b\01\00\00\1d\00\00\00\17\00\00\00\60\00\00\00\00\00\00\00\4e\00\00\00\00\00\00\00\0c\00\00\00\77\00\00\00\d4\00\00\00\d8\00\00\00"
  "\df\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\81\00\00\00\c9\00\00\00\1b\00\00\00\cc\00\00\00\fe\00\00\00\00\00\00\00\06\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00"
  "\00\00\00\00\00\00\00\00\3a\00\00\00\03\00\00\00\ea\00\00\00\00\00\00\00\00\00\00\00\bb\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\a9\00\00\00\00\00\00\00\9c\00\00\00\98\00\00\00"
  "\b1\00\00\00\d3\00\00\00\d5\00\00\00\3e\00\00\00\07\00\00\00\b2\00\00\00\2f\00\00\00\71\00\00\00\00\00\00\00\be\00\00\00\28\00\00\00\c5\00\00\00\c4\00\00\00"
)

;; Compare two bounded byte sequences. The caller has checked their lengths.
(func $names_equal (param $a i32) (param $b i32) (param $length i32) (result i32)
  (local $i i32)
  (block $equal
    (loop $bytes
      (br_if $equal (i32.eq (local.get $i) (local.get $length)))
      (if (i32.ne (i32.load8_u (i32.add (local.get $a) (local.get $i)))
                  (i32.load8_u (i32.add (local.get $b) (local.get $i))))
        (then (return (i32.const 0))))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $bytes)))
  (i32.const 1))

;; Look up a record with FNV-1a and bounded linear probing.
(func $names_lookup (param $address i32) (param $length i32)
    (param $slots i32) (param $capacity i32) (param $records i32) (result i32)
  (local $hash i32) (local $i i32) (local $slot i32)
  (local $entry i32) (local $record i32)
  (local.set $hash (i32.const -2128831035))
  (block $hashed
    (loop $hash_bytes
      (br_if $hashed (i32.eq (local.get $i) (local.get $length)))
      (local.set $hash (i32.mul
        (i32.xor (local.get $hash)
          (i32.load8_u (i32.add (local.get $address) (local.get $i))))
        (i32.const 16777619)))
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $hash_bytes)))
  (local.set $slot (i32.and (local.get $hash)
    (i32.sub (local.get $capacity) (i32.const 1))))
  (local.set $i (i32.const 0))
  (loop $probe
    (if (i32.eq (local.get $i) (local.get $capacity))
      (then (return (i32.const 0))))
    (local.set $entry (i32.load (i32.add (local.get $slots)
      (i32.shl (local.get $slot) (i32.const 2)))))
    (if (i32.eqz (local.get $entry)) (then (return (i32.const 0))))
    (local.set $record (i32.add (local.get $records)
      (i32.mul (i32.sub (local.get $entry) (i32.const 1)) (i32.const 12))))
    (if (i32.eq (i32.load offset=4 (local.get $record)) (local.get $length))
      (then
        (if (call $names_equal (local.get $address)
              (i32.load (local.get $record)) (local.get $length))
          (then (return (local.get $record))))))
    (local.set $slot (i32.and (i32.add (local.get $slot) (i32.const 1))
      (i32.sub (local.get $capacity) (i32.const 1))))
    (local.set $i (i32.add (local.get $i) (i32.const 1)))
    (br $probe))
  (i32.const 0))

;; Return a permanent primitive node, or zero for an unknown name.
(func $prim_lookup (param $address i32) (param $length i32) (result i32)
  (local $record i32)
  (local.set $record (call $names_lookup (local.get $address) (local.get $length)
    (i32.const 45172) (i32.const 512) (i32.const 38332)))
  (if (result i32) (local.get $record)
    (then (call $prim (i32.load offset=8 (local.get $record))))
    (else (i32.const 0))))

;; Return the canonical NUL-terminated name, or zero for a scalar tag.
;; The reverse table selects I for its shared I, ord, and chr tag.
(func $prim_name (param $tag i32) (result i32)
  (if (i32.ge_u (local.get $tag) (global.get $T_LAST_TAG))
    (then (return (i32.const 0))))
  (i32.load (i32.add (i32.const 37108)
    (i32.shl (local.get $tag) (i32.const 2)))))

;; Return a fixed service index, or -1 for an unknown name.
;; Library aliases use the same operation and arity as the canonical service.
(func $fixed_service_lookup (param $address i32) (param $length i32) (result i32)
  (local $record i32)
  (local.set $record (call $names_lookup (local.get $address) (local.get $length)
    (i32.const 47220) (i32.const 512) (i32.const 41764)))
  (if (result i32) (local.get $record)
    (then (i32.div_u (i32.sub (local.get $record) (i32.const 41764)) (i32.const 12)))
    (else (call $service_alias (local.get $address) (local.get $length)))))

;; Foreign.Storable uses these names for C-shaped scalar types. They remain
;; library service names in this runtime. The target uses signed eight-bit char,
;; signed eight-bit schar, unsigned eight-bit uchar, and sixteen-bit short.
;; Keep canonical service IDs so parsed aliases and serialized names agree.
;; These strings occupy 0xd100..0xd16d, outside the main immutable name tables.
(data (i32.const 0xd100)
  "peek_char\00poke_char\00peek_schar\00poke_schar\00peek_uchar\00poke_uchar\00"
  "peek_short\00poke_short\00peek_ushort\00poke_ushort\00")

;; This fallback is used only after the canonical hash lookup has failed.
;; Check length before comparing bytes, so a short input is never over-read.
(func $service_alias (param $address i32) (param $length i32) (result i32)
  (if (i32.eq (local.get $length) (i32.const 9))
    (then
      (if (call $names_equal (local.get $address) (i32.const 0xd100) (i32.const 9))
        (then (return (global.get $svc_peek_int8))))
      (if (call $names_equal (local.get $address) (i32.const 0xd10a) (i32.const 9))
        (then (return (global.get $svc_poke_int8))))))
  (if (i32.eq (local.get $length) (i32.const 10))
    (then
      (if (call $names_equal (local.get $address) (i32.const 0xd114) (i32.const 10))
        (then (return (global.get $svc_peek_int8))))
      (if (call $names_equal (local.get $address) (i32.const 0xd11f) (i32.const 10))
        (then (return (global.get $svc_poke_int8))))
      (if (call $names_equal (local.get $address) (i32.const 0xd12a) (i32.const 10))
        (then (return (global.get $svc_peek_uint8))))
      (if (call $names_equal (local.get $address) (i32.const 0xd135) (i32.const 10))
        (then (return (global.get $svc_poke_uint8))))
      (if (call $names_equal (local.get $address) (i32.const 0xd140) (i32.const 10))
        (then (return (global.get $svc_peek_int16))))
      (if (call $names_equal (local.get $address) (i32.const 0xd14b) (i32.const 10))
        (then (return (global.get $svc_poke_int16))))))
  (if (i32.eq (local.get $length) (i32.const 11))
    (then
      (if (call $names_equal (local.get $address) (i32.const 0xd156) (i32.const 11))
        (then (return (global.get $svc_peek_uint16))))
      (if (call $names_equal (local.get $address) (i32.const 0xd162) (i32.const 11))
        (then (return (global.get $svc_poke_uint16))))))
  (i32.const -1))

;; Return the scalar arity, or -1 for an invalid service index.
(func $fixed_service_arity (param $index i32) (result i32)
  (if (i32.ge_u (local.get $index) (global.get $service_count))
    (then (return (i32.const -1))))
  (i32.load offset=8 (i32.add (i32.const 41764)
    (i32.mul (local.get $index) (i32.const 12)))))

;; Return the NUL-terminated service name, or zero for an invalid index.
(func $fixed_service_name (param $index i32) (result i32)
  (if (i32.ge_u (local.get $index) (global.get $service_count))
    (then (return (i32.const 0))))
  (i32.load (i32.add (i32.const 41764)
    (i32.mul (local.get $index) (i32.const 12)))))
