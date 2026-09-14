module MicroHs.FFI(makeFFI, makeWasmFFI, hasWasmImports) where
import qualified Prelude(); import MHSPrelude
import Data.Char
import Data.List
import MicroHs.Desugar(LDef)
import MicroHs.Exp
import MicroHs.Expr
import MicroHs.Flags
import MicroHs.Ident
import MicroHs.Names
--import Debug.Trace

-- The export table has (internal-name, external-name, external-type)
makeFFI :: Flags -> [(Ident, Ident, CType, IsJavascript)] -> [IdentModule ]-> [[LDef]] -> (String, String)
makeFFI _ forExps exclude dss =
  let allImports = collectImports dss
      ffiImports = nubBy eq allImports
                 where eq (_, n, _, _) (_, n', _, _) = n == n'
      wrappers = [ t | (ImpWrapper, _, t, _) <- ffiImports]
      dynamics = [ t | (ImpDynamic, _, t, _) <- ffiImports]
      imps     = filter ((`notElem` exclude) . impModule) $ filter ((`notElem` runtimeFFI) . impName) ffiImports
      includes = jsincs ++ nub [ inc | (ImpStatic iincs _ _, _, _, _) <- imps, inc <- iincs ]
      jsincs   = if any isJS ffiImports then ["emscripten.h"] else []
        where isJS (ImpJS _, _, _, _) = True
              isJS _ = False
      mkSig (_, i, CType t, js) = let (as, ior) = getArrows t in mkExportSig js i as ior ++ ";"
      header = unlines
        ["#include <stdint.h>",
         "#if defined(__cplusplus)",
         "extern \"C\" {",
         "#endif",
         "void mhs_init(void);",
         intercalate "\n" $ map mkSig forExps,
         "#if defined(__cplusplus)",
         "}",
         "#endif"
        ]
  in
    checkWasmImports forExps allImports `seq`
    if not (null wrappers) || not (null dynamics) then mhsError "Unimplemented FFI feature" else
    (unlines $
      map (\ fn -> "#include \"" ++ fn ++ "\"") includes ++
      (if any (\ (_, _, _, js) -> js) forExps then
         ["#if defined(__EMSCRIPTEN__)",
          "#include \"emscripten.h\"",
          "#else",
          "#define EMSCRIPTEN_KEEPALIVE",
          "#endif"]
       else []) ++
      map mkHdr imps ++
      ["static const struct ffi_entry imp_table[] = {"] ++
      map mkEntry imps ++
      ["{ 0,0 }",
       "};",
       "const struct ffi_entry *xffi_table = imp_table;"
      ] ++
      ["static struct ffe_entry exp_table[] = {"] ++
      map mkExport forExps ++
      ["  { 0,0 }",
       "};",
       "struct ffe_entry *xffe_table = exp_table;",
       "\n"
      ] ++ zipWith mkExportWrapper [0..] forExps
    , header)

-- | Collect foreign imports from linked definitions.
collectImports :: [[LDef]] -> [(ImpEnt, String, EType, IdentModule)]
collectImports dss =
  [ (ie, n, t, mn) | ds <- dss, (_, d) <- ds, Lit (LForImp mn ie n (CType t)) <- [get d] ]
  where get (App _ a) = a
        get a = a

-- | Check whether definitions require WebAssembly imports.
hasWasmImports :: [[LDef]] -> Bool
hasWasmImports dss = any isWasm (collectImports dss)
  where isWasm (ImpWasm _ _, _, _, _) = True
        isWasm _ = False

-- | Reject conflicting import signatures and generated C names.
checkWasmImports :: [(Ident, Ident, CType, IsJavascript)] -> [(ImpEnt, String, EType, IdentModule)] -> ()
checkWasmImports exps imps =
  let signature t = let (as, r) = wasmTypes t in (map valueType as, valueType r)
      valueType "intptr_t" = "i32"
      valueType "uintptr_t" = "i32"
      valueType "int64_t" = "i64"
      valueType "uint64_t" = "i64"
      valueType s = s
      ws = [ (m, n, f, signature t) | (ImpWasm m n, f, t, _) <- imps ]
      conflicts = [ m ++ " " ++ n | (m, n, _, t) <- ws, (m', n', _, t') <- ws, m == m', n == n', t /= t' ]
      names = [ unIdent n | (_, n, _, _) <- exps ] ++
              concat [ [n, f, "mhs_" ++ f] | (ImpStatic _ _ n, f, _, _) <- imps ] ++
              [ "mhs_" ++ f | (ImpJS _, f, _, _) <- imps ]
      collisions = [ s | (_, _, f, _) <- ws, s <- [f, "mhs_" ++ f, "mhs_wasm_import_" ++ f], s `elem` names ]
  in case conflicts of
       n : _ -> mhsError $ "Conflicting foreign import wasm signatures: " ++ n
       [] -> case collisions of
               n : _ -> mhsError $ "foreign import wasm C name collision: " ++ n
               [] -> ()

-- | Return C scalar types for a WebAssembly function signature.
wasmTypes :: EType -> ([String], String)
wasmTypes t =
  let (as, ior) = getArrows t
      r = checkIO ior
      scalar a =
        case a of
          EVar i | unIdent i `elem` map ("Primitives." ++) ["Int", "Word", "Int64", "Word64", "Float", "Double"] -> cTypeName a
          _ -> errorMessage (getSLoc a) $ "Not a valid wasm scalar type: " ++ showEType a
      result = if isUnit r && not (eqEType r ior) then "void" else scalar r
  in (map scalar as, result)

-- | Emit standalone WAT dispatch, typed imports, and JavaScript metadata.
-- The linked combinator program retains its foreign names. Runtime service
-- indices start at 65536; the functions below receive relative indices.
-- Calls box scalar results but do not invoke the evaluator or collector.
makeWasmFFI :: [(Ident, Ident, CType, IsJavascript)] -> [[LDef]] -> (String, String, String)
makeWasmFFI exps dss =
  let allImports = collectImports dss
      imps = nubBy (\ a b -> impName a == impName b) $ filter direct allImports
      direct (ImpWasm _ _, _, _, _) = True
      direct (ImpJS _, _, _, _) = True
      direct _ = False
      invalid = [ n | (ie, n, _, _) <- allImports, not (allowed ie n) ]
      allowed (ImpWasm _ _) _ = True
      allowed (ImpJS _) _ = True
      allowed (ImpStatic _ _ _) n = n `elem` runtimeFFI
      allowed _ _ = False
      names = map impName imps
      addresses = scanl (\ p n -> p + length n + 1) 25165824 names
      entries = zip3 [0::Int ..] addresses imps
      number = show
      iconst n = "(i32.const " ++ number n ++ ")"
      same i = "(i32.eq (local.get $index) " ++ iconst i ++ ")"
      ffiName i = "$foreign_import_" ++ number i
      signature (_, _, ty, _) = wasmTypes ty
      importLine (i, _, imp@(ie, n, _, _)) =
        let (as, r) = signature imp
            (m, f) = case ie of
              ImpWasm modName field -> (modName, field)
              _ -> ("javascript", n)
        in "(import " ++ show m ++ " " ++ show f ++ " (func " ++ ffiName i ++
           concatMap (\ t -> " (param " ++ wasmValueType t ++ ")") as ++
           (if r == "void" then "" else " (result " ++ wasmValueType r ++ ")") ++ "))"
      nameData (_, p, (_, n, _, _)) =
        "(data " ++ iconst p ++ " \"" ++ n ++ "\\00\")"
      lookupCase (i, p, (_, n, _, _)) =
        "  (if (i32.eq (local.get $length) " ++ iconst (length n) ++ ")\n" ++
        "    (then (if (call $names_equal (local.get $name) " ++ iconst p ++ " (local.get $length))\n" ++
        "      (then (return " ++ iconst i ++ ")))))"
      arityCase (i, _, imp) =
        "  (if " ++ same i ++ " (then (return " ++ iconst (length (fst (signature imp))) ++ ")))"
      nameCase (i, p, _) = "  (if " ++ same i ++ " (then (return " ++ iconst p ++ ")))"
      invokeCase (i, _, imp) =
        let (as, r) = signature imp
            arg k t = " (call $" ++ wasmUnbox t ++ " (call $value_arg (local.get $args) " ++ iconst k ++ "))"
            call = "(call " ++ ffiName i ++ concat (zipWith arg [0::Int ..] as) ++ ")"
            result = if r == "void" then call ++ " (return (call $prim (global.get $T_I)))"
                     else "(return (call $" ++ wasmBox r ++ " " ++ call ++ "))"
        in "  (if " ++ same i ++ " (then " ++ result ++ "))"
      jsEntry (_, _, (ImpJS source, n, ty, _)) =
        let (as, r) = wasmTypes ty
        in ["{\"module\":\"javascript\",\"name\":" ++ jsonString n ++
            ",\"source\":" ++ jsonString source ++ ",\"parameters\":[" ++
            intercalate "," (map (jsonString . wasmScalarName) as) ++
            "],\"result\":" ++ jsonString (wasmScalarName r) ++ "}"]
      jsEntry _ = []
      arities = [ impName imp | imp <- imps, length (fst (signature imp)) > 6 ]
      code = unlines $
        [";; Generated scalar foreign calls. This is a WAT module fragment.",
         ";; Arguments are boxed values in weak head normal form. Calls do not collect."] ++
        map nameData entries ++
        ["(func $foreign_lookup (param $name i32) (param $length i32) (result i32)"] ++
        map lookupCase entries ++ ["  (i32.const -1))",
        "(func $foreign_arity (param $index i32) (result i32)"] ++
        map arityCase entries ++ ["  (call $fail (i32.const 38)) (unreachable))",
        "(func $foreign_name (param $index i32) (result i32)"] ++
        map nameCase entries ++ ["  (call $fail (i32.const 38)) (unreachable))",
        "(func $foreign_invoke (param $index i32) (param $args i32) (result i32)"] ++
        map invokeCase entries ++ ["  (call $fail (i32.const 38)) (unreachable))"]
      metadata = "{\"version\":1,\"bindings\":[" ++ intercalate "," (concatMap jsEntry entries) ++ "]}\n"
  in checkWasmImports exps allImports `seq`
     if not (null exps) then mhsError "foreign export is not supported by standalone WAT output" else
     case invalid of
       n : _ -> mhsError $ "standalone WAT output cannot call C foreign import: " ++ n
       [] -> case arities of
         n : _ -> mhsError $ "standalone WAT foreign import has more than 6 arguments: " ++ n
         [] -> if last addresses > 33554432 then mhsError "standalone WAT foreign names exceed 8 MiB" else
               (code, unlines (map importLine entries), metadata)

-- | Map scalar C spellings to the WebAssembly value types used by both backends.
wasmValueType :: String -> String
wasmValueType t = case t of
  "intptr_t" -> "i32"
  "uintptr_t" -> "i32"
  "int64_t" -> "i64"
  "uint64_t" -> "i64"
  "float" -> "f32"
  "double" -> "f64"
  _ -> mhsError $ "Not a WebAssembly value type: " ++ t

-- | Select the checked scalar accessor for a foreign argument.
wasmUnbox :: String -> String
wasmUnbox t = case wasmValueType t of
  "i32" -> "ival"
  "i64" -> "i64val"
  "f32" -> "fval"
  _ -> "dval"

-- | Select the graph constructor for a foreign result.
wasmBox :: String -> String
wasmBox t = case wasmValueType t of
  "i32" -> "int"
  "i64" -> "int64"
  "f32" -> "float"
  _ -> "double"

-- | Preserve signedness in JavaScript metadata. WebAssembly itself has no unsigned value types.
wasmScalarName :: String -> String
wasmScalarName t = case t of
  "intptr_t" -> "Int"
  "uintptr_t" -> "Word"
  "int64_t" -> "Int64"
  "uint64_t" -> "Word64"
  "float" -> "Float"
  "double" -> "Double"
  "void" -> "Unit"
  _ -> mhsError $ "Not a WebAssembly scalar type: " ++ t

-- | Quote JSON source text without Haskell-specific string escapes.
jsonString :: String -> String
jsonString s = '"' : concatMap escape s ++ "\""
  where
    escape '"' = "\\\""
    escape '\\' = "\\\\"
    escape c | ord c < 32 = "\\u00" ++ [intToDigit (ord c `div` 16), intToDigit (ord c `mod` 16)]
             | otherwise = [c]

mkExportSig :: IsJavascript -> Ident -> [EType] -> EType -> String
mkExportSig js n as ior =
  let outT = expTypeName js $ checkIO ior
      ins = zipWith (\ i a -> expTypeName js a ++ " _x" ++ show i) [1::Int ..] as
   in outT ++ " " ++ unIdent n ++ "(" ++ intercalate ", " ins ++ ")"

mkExport :: (Ident, Ident, CType, IsJavascript) -> String
mkExport (i, _, _, _) = "  { \"" ++ unIdent i ++ "\", 0 },"

mkExportWrapper :: Int -> (Ident, Ident, CType, IsJavascript) -> String
mkExportWrapper no (_, n, CType t, js) = unlines $
  let (as, ior) = getArrows t
      r = checkIO ior
      outT = expTypeName js r
      arg k a = "  mhs_from_" ++ expTypeHsName js a ++ "(ffe_alloc(), 0, _x" ++ show k ++ "); ffe_apply();"
      eval = if eqEType r ior then "ffe_eval()" else "ffe_exec()"
  in  [(if js then "EMSCRIPTEN_KEEPALIVE " else "") ++ mkExportSig js n as ior ++ " {",
       "  gc_check(" ++ show (2 * length as + 4) ++ ");",
       "  ffe_push(xffe_table[" ++ show no ++ "].ffe_value);" ]
      ++ zipWith arg [1::Int ..] as ++
      if isUnit r then
        [ "  (void)" ++ eval ++ ";",
          "  ffe_pop();",
          "}"
        ]
       else
        [ "  " ++ outT ++ " _res = mhs_to_" ++ expTypeHsName js r ++ "(" ++ eval ++ ", -1);",
          "  ffe_pop();",
          "  return _res;",
          "}"
        ]

impName :: (ImpEnt, String, EType, IdentModule) -> String
impName (_, s, _, _) = s

impModule :: (ImpEnt, String, EType, IdentModule) -> IdentModule
impModule (_, _, _, m) = m

mkEntry :: (ImpEnt, String, EType, IdentModule) -> String
mkEntry (ImpStatic _ IFunc  _, f, t, _) = "{ \"" ++ f ++ "\", " ++ show (arity t) ++ ", mhs_" ++ f ++ "},"
mkEntry (ImpStatic _ IPtr   _, f, _, _) = "{ \"&" ++ f ++ "\", 0, mhs_addr_" ++ f ++ "},"
mkEntry (ImpStatic _ IValue _, f, _, _) = "{ \"" ++ f ++ "\", 0, mhs_" ++ f ++ "},"
mkEntry (ImpJS _,              f, t, _) = "{ \"" ++ f ++ "\", " ++ show (arity t) ++ ", mhs_" ++ f ++ "},"
mkEntry (ImpWasm _ _,          f, t, _) = "{ \"" ++ f ++ "\", " ++ show (arity t) ++ ", mhs_" ++ f ++ "},"
mkEntry _ = undefined

mkMhsFun :: String -> String -> String
mkMhsFun fn body = "from_t mhs_" ++ fn ++ "(int s) { " ++ body ++ "; }"

checkIO :: EType -> EType
checkIO iot =
  case dropApp identIO iot of
    Nothing -> iot -- errorMessage (getSLoc iot) $ "foreign return type must be IO: " ++ showEType iot
    Just t  -> t

dropApp :: Ident -> EType -> Maybe EType
dropApp i (EApp (EVar i') t) | i == i' = Just t
dropApp _ _ = Nothing

isUnit :: EType -> Bool
isUnit (EVar unit) = unit == identUnit
isUnit _ = False

mkRet :: HasCallStack => EType -> Int -> String -> String
mkRet t n call = "mhs_from_" ++ cTypeHsName t ++ "(s, " ++ show n ++ ", " ++ call ++ ")"

mkArg :: EType -> Int -> String
mkArg t i = "mhs_to_" ++ cTypeHsName t ++ "(s, " ++ show i ++ ")"

mkJSArg :: EType -> Int -> String
mkJSArg t i = "mhs_to_" ++ jsTypeName t ++ "(s, " ++ show i ++ ")"

mkHdr :: (ImpEnt, String, EType, IdentModule) -> String
mkHdr (ImpStatic _ IPtr fn, f, iot, _) =
  let r = checkIO iot
      (s, _) =
        case dropApp identPtr r of
          Just t  -> ("", t)
          Nothing ->
            case dropApp identFunPtr r of
              Just t  -> ("(HsFunPtr)", t)
              Nothing -> errorMessage (getSLoc r) "foreign & must be Ptr/FunPtr"
      body = "return " ++ mkRet r 0 (s ++ "&" ++ fn)
  in  mkMhsFun ("addr_" ++ f) body
mkHdr (ImpStatic _ IFunc fn, f, t, _) =
  let (as, ior) = getArrows t
      r = checkIO ior
      len = length as
      call = fn ++ "(" ++ intercalate ", " (zipWith mkArg as [0..]) ++ ")"
      fcall =
        if isUnit r then
          call ++ "; return mhs_from_Unit(s, " ++ show len ++ ")"
        else
          "return " ++ mkRet r len call
  in  mkMhsFun f fcall
mkHdr (ImpStatic _ IValue val, f, t, _) =
  let (as, ior) = getArrows t
      r = checkIO ior
      len = length as
      call = expand val
      expand [] = []
      expand ('$':c:cs) | isDigit c =
        let n = digitToInt c - 1
        in mkArg (as !! n) n ++ expand cs
      expand (c:cs) = c : expand cs
      fcall =
        if isUnit r then
          call ++ "; return mhs_from_Unit(s, " ++ show len ++ ")"
        else
          "return " ++ mkRet r len call
  in  mkMhsFun f fcall
mkHdr (ImpJS s, f, ty, _) =
  let (as, ior) = getArrows ty
      rt = checkIO ior
      jsr = jsTypeNameR rt
      n = length as
      args = concat $ zipWith arg as [0..]
      arg t i = ", " ++ mkJSArg t i
      call = "EM_ASM" ++
             (if isUnit rt then "" else '_':jsr) ++
             "({ " ++ s ++ " }" ++ args ++ ")"
      fcall =
        if isUnit rt then
          call ++ "; return mhs_from_Unit(s, " ++ show n ++ ")"
        else
          "return " ++ mkRet rt n call
  in  mkMhsFun f fcall
mkHdr (ImpWasm m n, f, ty, mn) =
  let (as, r) = wasmTypes ty
      fn = "mhs_wasm_import_" ++ f
      args = if null as then "void" else intercalate ", " as
      declaration = "extern " ++ r ++ " " ++ fn ++ "(" ++ args ++ ") " ++
                    "__attribute__((import_module(" ++ show m ++ "), import_name(" ++ show n ++ ")));"
  in unlines [ "#if !defined(__wasm32__)",
               "#error foreign import wasm requires a wasm32 target",
               "#endif",
               declaration,
               mkHdr (ImpStatic [] IFunc fn, f, ty, mn)
             ]
mkHdr _ = undefined

arity :: EType -> Int
arity = length . fst . getArrows

-- Use to construct 'foreign import/export ccall' wrapper.
cTypeHsName :: HasCallStack => EType -> String
cTypeHsName (EApp (EVar ptr) _t) | ptr == identPtr = "Ptr"
                                 | ptr == identFunPtr = "FunPtr"
cTypeHsName (EVar i) | Just c <- lookup (unIdent i) cHsTypes = c
cTypeHsName t = errorMessage (getSLoc t) $ "Not a valid C type: " ++ showEType t

cHsTypes :: [(String, String)]
cHsTypes =
  [ ("Primitives.Float",  "Float")
  , ("Primitives.Double", "Double")
  , ("Primitives.Int",    "Int")
  , ("Primitives.Int64",  "Int64")
  , ("Primitives.Word",   "Word")
  , ("Primitives.Word64", "Word64")
  , ("()",                "Unit")
  , ("System.IO.Handle",  "Ptr")
  ]

-- Foreign export type names; a javascript export also allows Bool.
expTypeName :: IsJavascript -> EType -> String
expTypeName True (EVar i) | unIdent i == "Data.Bool_Type.Bool" = "int"
expTypeName _ t = cTypeName t

expTypeHsName :: IsJavascript -> EType -> String
expTypeHsName True (EVar i) | unIdent i == "Data.Bool_Type.Bool" = "Bool"
expTypeHsName _ t = cTypeHsName t

-- Use to construct 'foreign export ccall' signature.
cTypeName :: EType -> String
cTypeName (EApp (EVar ptr) _t) | ptr == identPtr = "void*"
cTypeName (EVar i) | Just c <- lookup (unIdent i) cTypes = c
cTypeName t = errorMessage (getSLoc t) $ "Not a valid C type: " ++ showEType t

cTypes :: [(String, String)]
cTypes =
  [ ("Primitives.Float",  "float")
  , ("Primitives.Double", "double")
  , ("Primitives.Int",    "intptr_t")   -- value_t
  , ("Primitives.Int64",  "int64_t")
  , ("Primitives.Word",   "uintptr_t")  -- uvalue_t
  , ("Primitives.Word64", "uint64_t")
  , ("()",                "void")
  , ("System.IO.Handle",  "void*")
  ]

-- Use to construct 'foreign import javascript' return value wrapper.
jsTypeNameR :: EType -> String
jsTypeNameR (EApp (EVar ptr) _) | ptr == identPtr = "PTR"
jsTypeNameR (EVar i) | Just c <- lookup (unIdent i) jsTypesR = c
jsTypeNameR t = errorMessage (getSLoc t) $ "Not a valid Javascript return type: " ++ showEType t

jsTypesR :: [(String, String)]
jsTypesR =
  [ ("Primitives.Int",    "INT")
  , ("Primitives.Double", "DOUBLE")
  , ("Primitives.Float",  "DOUBLE")
  ]

-- Use to construct 'foreign import javascript' argument wrapper.
jsTypeName :: EType -> String
jsTypeName (EApp (EVar ptr) _) | ptr == identPtr = "Ptr"
jsTypeName (EVar i) | Just c <- lookup (unIdent i) jsTypes = c
jsTypeName t = errorMessage (getSLoc t) $ "Not a valid Javascript argument type: " ++ showEType t

jsTypes :: [(String, String)]
jsTypes =
  [ ("Primitives.Int",    "Int")
  , ("Primitives.Double", "Double")
  , ("Primitives.Float",  "Float")
  ]

-- These are already in the runtime
runtimeFFI :: [String]
runtimeFFI = [
  "GETRAW", "GETTIMEMICRO", "GETBOOTTIMEMICRO", "acos", "add_FILE", "add_fd", "open", "add_utf8", "add_buf", "add_crlf",
  "asin", "atan", "atan2", "calloc", "closeb",
  "cos", "exp", "flushb", "fopen", "free", "getb", "getenv", "islinux", "ismacos", "iswindows", "log", "malloc",
  "md5Array", "md5BFILE", "md5String", "memcpy", "memmove", "realloc", "strlen", "strcpy",
  "putb", "sin", "sqrt", "system", "tan", "tmpname", "ungetb", "remove",
  "acosf", "asinf", "atanf", "atan2f", "cosf", "expf", "logf", "sinf", "sqrtf", "tanf",
  "scalbn", "scalbnf", "pow", "powf",
  "js_debug", "js_eval_run", "js_eval_call", "js_set_haskellCallback",
  "readb", "writeb",
  "peekPtr", "pokePtr", "pokeWord", "peekWord",
  "add_lz77_compressor", "add_lz77_decompressor",
  "add_lzma_compressor", "add_lzma_decompressor",
  "add_rle_compressor", "add_rle_decompressor",
  "add_base64_encoder", "add_base64_decoder",
  "add_bwt_compressor", "add_bwt_decompressor",
  "peek_uint8", "poke_uint8", "peek_uint16", "poke_uint16", "peek_uint32", "poke_uint32", "peek_uint64", "poke_uint64",
  "peek_int8", "poke_int8", "peek_int16", "poke_int16", "peek_int32", "poke_int32", "peek_int64", "poke_int64",
  "peek_char", "poke_char", "peek_schar", "poke_schar", "peek_uchar", "poke_uchar",
  "peek_ushort", "poke_ushort", "peek_short", "poke_short",
  "peek_uint", "poke_uint", "peek_int", "poke_int",
  "peek_ulong", "poke_ulong", "peek_long", "poke_long",
  "peek_ullong", "poke_ullong", "peek_llong", "poke_llong",
  "peek_size_t", "poke_size_t",
  "peek_flt32", "poke_flt32",
  "peek_flt64", "poke_flt64",
  "sizeof_char", "sizeof_short", "sizeof_int", "sizeof_long", "sizeof_llong", "sizeof_size_t",
  "opendir", "closedir", "readdir", "c_d_name", "chdir", "mkdir", "getcwd",
  "getcpu",
  "get_mem", "openb_rd_mem", "openb_wr_mem",
  "new_mpz", "mpz_abs", "mpz_add", "mpz_and", "mpz_cmp", "mpz_get_d", "mpz_get_f",
  "mpz_get_si", "mpz_init_set_si", "mpz_init_set_ui",
  "mpz_ior",
  "mpz_mul", "mpz_mul_2exp", "mpz_neg", "mpz_popcount", "mpz_sub", "mpz_fdiv_q_2exp",
  "mpz_tdiv_qr", "mpz_tstbit", "mpz_xor",
  "mpz_get_si64", "mpz_init_set_si64", "mpz_init_set_ui64",
  "mpz_log2",
  "want_gmp",
  "want_imath",
  "gettimeofday",
  "EOK", "E2BIG", "EACCES", "EADDRINUSE", "EADDRNOTAVAIL", "EADV", "EAFNOSUPPORT", "EAGAIN",
  "EALREADY", "EBADF", "EBADMSG", "EBADRPC", "EBUSY", "ECHILD", "ECOMM", "ECONNABORTED",
  "ECONNREFUSED", "ECONNRESET", "EDEADLK", "EDESTADDRREQ", "EDIRTY", "EDOM", "EDQUOT",
  "EEXIST", "EFAULT", "EFBIG", "EFTYPE", "EHOSTDOWN", "EHOSTUNREACH", "EIDRM", "EILSEQ",
  "EINPROGRESS", "EINTR", "EINVAL", "EIO", "EISCONN", "EISDIR", "ELOOP", "EMFILE", "EMLINK",
  "EMSGSIZE", "EMULTIHOP", "ENAMETOOLONG", "ENETDOWN", "ENETRESET", "ENETUNREACH",
  "ENFILE", "ENOBUFS", "ENODATA", "ENODEV", "ENOENT", "ENOEXEC", "ENOLCK", "ENOLINK",
  "ENOMEM", "ENOMSG", "ENONET", "ENOPROTOOPT", "ENOSPC", "ENOSR", "ENOSTR", "ENOSYS",
  "ENOTBLK", "ENOTCONN", "ENOTDIR", "ENOTEMPTY", "ENOTSOCK", "ENOTSUP", "ENOTTY", "ENXIO",
  "EOPNOTSUPP", "EPERM", "EPFNOSUPPORT", "EPIPE", "EPROCLIM", "EPROCUNAVAIL",
  "EPROGMISMATCH", "EPROGUNAVAIL", "EPROTO", "EPROTONOSUPPORT", "EPROTOTYPE",
  "ERANGE", "EREMCHG", "EREMOTE", "EROFS", "ERPCMISMATCH", "ERREMOTE", "ESHUTDOWN",
  "ESOCKTNOSUPPORT", "ESPIPE", "ESRCH", "ESRMNT", "ESTALE", "ETIME", "ETIMEDOUT",
  "ETOOMANYREFS", "ETXTBSY", "EUSERS", "EWOULDBLOCK", "EXDEV",
  "errno",
  "strerror_r",
  "environ",
  "get_executable_path",
  "setenv", "unsetenv",
  "set_permissions", "get_permissions",
  "F_SETFL", "O_NONBLOCK", "SOL_SOCKET", "SO_DEBUG", "SO_ERROR", "SO_REUSEADDR", "SO_TYPE",
  "accept", "bind", "close", "connect", "fcntl", "getsockopt", "listen", "recv", "send", "setsockopt", "socket"
  ]

{-
-- lib/ modules that use foreign import
libImports :: [String]
libImports = [
  "Data.Integer_Type",
  "Data.Integer.Internal",
  "System.Process",
  "System.Environment",
  "System.Compress.ByteString",
  "System.Compress",
  "System.IO.TimeMilli",
  "System.IO.Open",
  "System.IO.Transducers",
  "System.IO.MD5",
  "System.IO.Base",
  "System.IO.StringHandle",
  "System.IO.Internal",
  "System.IO.Serialize",
  "System.CPUTime",
  "System.Directory",
  "System.Cmd",
  "Data.ByteString",
  "Data.Double",
  "Data.Float",
  "Foreign.Marshal.Utils",
  "Foreign.Marshal.Alloc",
  "Foreign.Storable",
  "Foreign.C.Error",
  "Primitives"
  ]
-}
