{-# LANGUAGE LambdaCase #-}
-- | Render a MicroHs combinator term as a binary tree and (for pure terms)
-- as a Barker-Iota program.
--
-- Pipeline:
--   1. parse a @-ddump-combinator@ dump (fully-parenthesized applications),
--   2. inline top-level references from a chosen root into one term,
--   3. classify leaves: structural combinator | opaque primitive box,
--   4. for primitive-free terms, expand every combinator to pure S/K via
--      bracket abstraction of its reduction rule, then S/K -> their iota trees,
--   5. ASCII-render the trees and emit the 0/1 iota string.
module Main (main) where

import Data.Char (isSpace)
import Data.List (intercalate)
import qualified Data.Map.Strict as M
import System.Environment (getArgs)

------------------------------------------------------------------------
-- Terms (binary application trees with string leaves)

data Tm = Ap Tm Tm | Lf String deriving (Eq, Show)

size :: Tm -> Int
size (Lf _)   = 1
size (Ap a b) = 1 + size a + size b

------------------------------------------------------------------------
-- Tokenizer + parser for the -ddump-combinator format.

data Tok = LP | RP | Atom String deriving (Eq, Show)

tokenize :: String -> [Tok]
tokenize [] = []
tokenize (c:cs)
  | c == '('  = LP : tokenize cs
  | c == ')'  = RP : tokenize cs
  | isSpace c = tokenize cs
  | c == '"'  = let (s, r) = str cs in Atom ('"' : s) : tokenize r
  | otherwise = let (a, r) = span isAtom (c:cs) in Atom a : tokenize r
  where
    isAtom x = not (isSpace x) && x /= '(' && x /= ')'
    str ('\\':y:ys) = let (s, r) = str ys in ('\\':y:s, r)
    str ('"':ys)    = ("\"", ys)
    str (y:ys)      = let (s, r) = str ys in (y:s, r)
    str []          = ("", [])

-- expr := Atom | '(' expr+ ')'   (applications are left-associative)
parseExpr :: [Tok] -> (Tm, [Tok])
parseExpr (Atom a : ts) = (Lf a, ts)
parseExpr (LP : ts)     =
  let (e, ts') = parseExpr ts
      (e', ts'') = apps e ts'
  in case ts'' of
       (RP : r) -> (e', r)
       _        -> error "parseExpr: expected )"
parseExpr ts = error ("parseExpr: unexpected " ++ show (take 4 ts))

apps :: Tm -> [Tok] -> (Tm, [Tok])
apps acc ts@(RP:_) = (acc, ts)
apps acc []        = (acc, [])
apps acc ts        = let (e, ts') = parseExpr ts in apps (Ap acc e) ts'

-- Parse a whole dump into (name, term) pairs.  Continuation lines (the
-- pretty-printer may wrap wide terms) start with whitespace and are joined.
parseDump :: String -> [(String, Tm)]
parseDump = map parseDef . chunks . filter (not . blank) . lines
  where
    blank = all isSpace
    chunks [] = []
    chunks (l:ls) = let (cont, rest) = span (isSpace . headOr ' ') ls
                    in unwords (l : cont) : chunks rest
    headOr d s = if null s then d else head s
    parseDef s = case breakOn " = " s of
      Just (nm, rhs) -> (trim nm, fst (apps' (tokenize rhs)))
      Nothing        -> error ("parseDump: no '=' in: " ++ s)
    apps' ts = let (e, ts') = parseExpr ts in apps e ts'

breakOn :: String -> String -> Maybe (String, String)
breakOn pat = go ""
  where
    go acc s@(c:cs)
      | pat `isPrefixOf'` s = Just (reverse acc, drop (length pat) s)
      | otherwise           = go (c:acc) cs
    go _ [] = Nothing
    isPrefixOf' p x = take (length p) x == p

trim :: String -> String
trim = f . f where f = reverse . dropWhile isSpace

------------------------------------------------------------------------
-- The combinator zoo, as reduction rules expressed as lambda terms.

data Lam = V String | App Lam Lam | Lam String Lam

infixl 9 #
(#) :: Lam -> Lam -> Lam
(#) = App

lam :: [String] -> Lam -> Lam
lam vs b = foldr Lam b vs

-- name -> its defining lambda (from the reduction rules in Abstract.hs / runtime)
zoo :: M.Map String Lam
zoo = M.fromList
  [ ("I",   lam ["x"] (v"x"))
  , ("B",   lam ["x","y","z"] (v"x" # (v"y" # v"z")))
  , ("C",   lam ["x","y","z"] (v"x" # v"z" # v"y"))
  , ("A",   lam ["x","y"] (v"y"))
  , ("U",   lam ["x","y"] (v"y" # v"x"))
  , ("Z",   lam ["x","y","z"] (v"x" # v"y"))
  , ("P",   lam ["x","y","z"] (v"z" # v"x" # v"y"))
  , ("R",   lam ["x","y","z"] (v"y" # v"z" # v"x"))
  , ("O",   lam ["x","y","z","w"] (v"w" # v"x" # v"y"))
  , ("S'",  lam ["x","y","z","w"] (v"x" # (v"y" # v"w") # (v"z" # v"w")))
  , ("B'",  lam ["x","y","z","w"] (v"x" # v"y" # (v"z" # v"w")))
  , ("C'",  lam ["x","y","z","w"] (v"x" # (v"y" # v"w") # v"z"))
  , ("C'B", lam ["x","y","z","w"] (v"x" # v"z" # (v"y" # v"w")))
  , ("K2",  lam ["x","y","z"] (v"x"))
  , ("K3",  lam ["x","y","z","w"] (v"x"))
  , ("K4",  lam ["x","y","z","w","u"] (v"x"))
  , ("J",   lam ["x","y","z"] (v"z" # v"x"))
    -- Curry's fixed-point combinator; finite lambda -> finite SK term.
  , ("Y",   Lam "f" (App yh yh))
  ]
  where yh = Lam "x" (v"f" # (v"x" # v"x"))

v :: String -> Lam
v = V

-- Structural combinators we recognise (atomic S,K plus the zoo).
isComb :: String -> Bool
isComb s = s == "S" || s == "K" || M.member s zoo

------------------------------------------------------------------------
-- Bracket abstraction: Lam -> SKI.

compileLam :: Lam -> Lam
compileLam (Lam x e) = abstract x (compileLam e)
compileLam (App a b) = App (compileLam a) (compileLam b)
compileLam e@(V _)   = e

abstract :: String -> Lam -> Lam
abstract x (V y) | x == y    = V "I"
                 | otherwise = V "K" # V y
abstract x (App a b)         = V "S" # abstract x a # abstract x b
abstract x e@(Lam _ _)       = abstract x (compileLam e)

lamToTm :: Lam -> Tm
lamToTm (V s)     = Lf s
lamToTm (App a b) = Ap (lamToTm a) (lamToTm b)
lamToTm (Lam _ _) = error "lamToTm: residual lambda"

------------------------------------------------------------------------
-- Expand a combinator term to pure S/K, then to an iota tree.

-- SK(I) form of a single combinator (S,K atomic; I -> S K K).
combSK :: String -> Tm
combSK "S" = Lf "S"
combSK "K" = Lf "K"
combSK "I" = Ap (Ap (Lf "S") (Lf "K")) (Lf "K")
combSK n   = case M.lookup n zoo of
  Just l  -> elimI (lamToTm (compileLam l))
  Nothing -> error ("combSK: unknown combinator " ++ n)

elimI :: Tm -> Tm
elimI (Lf "I")  = Ap (Ap (Lf "S") (Lf "K")) (Lf "K")
elimI (Lf s)    = Lf s
elimI (Ap a b)  = Ap (elimI a) (elimI b)

-- Whole term -> pure S/K term.  Fails on any non-combinator leaf.
toSK :: Tm -> Either String Tm
toSK (Ap a b) = Ap <$> toSK a <*> toSK b
toSK (Lf s)
  | isComb s  = Right (combSK s)
  | otherwise = Left s

-- iota trees for S and K (leaves are the iota combinator, "1").
iK, iS :: Tm
iK = ap [iI, iI, iI, iI]          -- K = i(i(i i))
  where ap = foldr1 Ap; iI = Lf "1"
iS = Ap (Lf "1") iK               -- S = i K

skToIota :: Tm -> Tm
skToIota (Lf "S") = iS
skToIota (Lf "K") = iK
skToIota (Lf s)   = error ("skToIota: not S/K: " ++ s)
skToIota (Ap a b) = Ap (skToIota a) (skToIota b)

-- 0/1 prefix encoding: Ap -> '0', leaf "1" -> '1'.
encodeIota :: Tm -> String
encodeIota (Lf "1")  = "1"
encodeIota (Lf s)    = error ("encodeIota: stray leaf " ++ s)
encodeIota (Ap a b)  = '0' : encodeIota a ++ encodeIota b

------------------------------------------------------------------------
-- Inlining references from a root.

-- does leaf `x` occur in the term?
occurs :: String -> Tm -> Bool
occurs x (Lf s)   = s == x
occurs x (Ap a b) = occurs x a || occurs x b

-- naive bracket abstraction over a leaf variable: \x. t  in S/K/I
absTm :: String -> Tm -> Tm
absTm x (Lf s) | s == x    = Lf "I"
               | otherwise = Ap (Lf "K") (Lf s)
absTm x (Ap a b)           = Ap (Ap (Lf "S") (absTm x a)) (absTm x b)

inline :: M.Map String Tm -> String -> Tm
inline defs0 root = go [] (Lf root)
  where
    -- MicroHs leaves top-level recursion as self-referential defs (f = ...f...).
    -- Rewrite each into a finite Y-term so the result is a tree, not a cycle.
    defs = M.mapWithKey ywrap defs0
    ywrap name body | occurs name body = Ap (Lf "Y") (absTm name body)
                    | otherwise        = body
    go stack (Ap a b) = Ap (go stack a) (go stack b)
    go stack (Lf s)
      | s `elem` stack             = Lf ("<rec:" ++ s ++ ">")  -- mutual-cycle guard
      | Just t <- M.lookup s defs  = go (s:stack) t
      | otherwise                  = Lf s                       -- comb / prim / lit

------------------------------------------------------------------------
-- Display.

-- Pretty leaf label for the combinator tree.
label :: String -> String
label s
  | isComb s                 = s
  | take 1 s `elem` litPfx   = "<" ++ s ++ ">"
  | head s == '<'            = s                -- rec marker
  | otherwise                = "<" ++ s ++ ">"  -- primitive box
  where litPfx = ["#","%","&","\"","'"]

-- s-expression with display labels: leaf -> label, node -> (L R).
sexpTm :: Tm -> String
sexpTm (Lf s)   = label s
sexpTm (Ap a b) = "(" ++ sexpTm a ++ " " ++ sexpTm b ++ ")"

drawWith :: (String -> String) -> Tm -> String
drawWith lbl = unlines . go
  where
    go (Lf s)   = [lbl s]
    go (Ap a b) = "@" : kids [a, b]
    kids []     = []
    kids [t]    = shift "└─ " "   " (go t)
    kids (t:ts) = shift "├─ " "│  " (go t) ++ kids ts
    shift f o (x:xs) = (f ++ x) : map (o ++) xs
    shift _ _ []     = []

-- iota tree: nodes '0', leaves '1'
drawIota :: Tm -> String
drawIota = drawWith id

------------------------------------------------------------------------

-- iota 0/1 string, or Nothing if the term has primitive leaves.
iotaString :: Tm -> Either String String
iotaString term = encodeIota . skToIota <$> toSK term

main :: IO ()
main = do
  args <- getArgs
  case args of
    ["sexp", p, r] -> do                         -- combinator tree as s-expr
      defs <- M.fromList . parseDump <$> readFile p
      putStrLn (sexpTm (inline defs r))
    ["isexp", p, r] -> do                        -- iota tree as s-expr (0/1 labels)
      defs <- M.fromList . parseDump <$> readFile p
      case iotaTree (inline defs r) of
        Right t  -> putStrLn (sexpRaw t)
        Left bad -> error ("impure: " ++ bad)
    ["sk", p, r] -> do                           -- pure S/K tree as s-expr
      defs <- M.fromList . parseDump <$> readFile p
      case toSK (inline defs r) of
        Right t  -> putStrLn (sexpRaw t)
        Left bad -> error ("impure: " ++ bad)
    ["iota", p, r] -> do                         -- just the 0/1 string
      defs <- M.fromList . parseDump <$> readFile p
      case iotaString (inline defs r) of
        Right s  -> putStrLn s
        Left bad -> putStrLn ("IMPURE " ++ bad)
    _ -> asciiMain args

-- raw s-expr (no relabelling), used for the 1/0 iota tree
sexpRaw :: Tm -> String
sexpRaw (Lf s)   = s
sexpRaw (Ap a b) = "(" ++ sexpRaw a ++ " " ++ sexpRaw b ++ ")"

iotaTree :: Tm -> Either String Tm
iotaTree term = skToIota <$> toSK term

asciiMain :: [String] -> IO ()
asciiMain args = do
  (path, root, maxTree) <- case args of
    [p, r]    -> pure (p, r, 64)
    [p, r, m] -> pure (p, r, read m)
    _ -> error "usage: iota [sexp|isexp|iota] DUMPFILE ROOTNAME [maxIotaTreeNodes]"
  defs <- M.fromList . parseDump <$> readFile path
  let term = inline defs root
  putStrLn ("== combinator term: " ++ root ++ "  (" ++ show (size term) ++ " nodes) ==")
  putStr (drawWith label term)
  putStrLn ""
  case toSK term of
    Left bad -> putStrLn ("Not pure-combinatory: contains primitive/foreign leaf "
                          ++ show bad ++ " -> no iota encoding (render as box).")
    Right sk -> do
      let iota = skToIota sk
          str  = encodeIota iota
      putStrLn ("== iota program (" ++ show (length str) ++ " symbols, "
                ++ show (size iota) ++ " tree nodes) ==")
      putStrLn str
      if size iota <= maxTree
        then putStrLn "" >> putStrLn "== iota tree ==" >> putStr (drawIota iota)
        else putStrLn ("(iota tree suppressed; " ++ show (size iota)
                       ++ " nodes > " ++ show maxTree ++ ")")
  pure ()
