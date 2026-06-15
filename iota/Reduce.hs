-- | Combinator graph reducer with a step trace.  Reads a -ddump-combinator dump,
-- reduces a chosen definition in normal order (unfolding definitions on demand and
-- firing combinator rules), and prints every step.
module Main (main) where

import Data.Char (isSpace)
import Data.List (foldl', isInfixOf)
import qualified Data.Map.Strict as M
import System.Environment (getArgs)

data Tm = Ap Tm Tm | Lf String deriving (Eq)

------------------------------------------------------------ parsing (as in Iota.hs)
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

parseExpr :: [Tok] -> (Tm, [Tok])
parseExpr (Atom a : ts) = (Lf a, ts)
parseExpr (LP : ts)     =
  let (e, ts') = parseExpr ts
      (e', ts'') = apps e ts'
  in case ts'' of (RP : r) -> (e', r); _ -> error "parseExpr: expected )"
parseExpr ts = error ("parseExpr: unexpected " ++ show (take 4 ts))

apps :: Tm -> [Tok] -> (Tm, [Tok])
apps acc ts@(RP:_) = (acc, ts)
apps acc []        = (acc, [])
apps acc ts        = let (e, ts') = parseExpr ts in apps (Ap acc e) ts'

parseDump :: String -> M.Map String Tm
parseDump = M.fromList . map parseDef . chunks . filter (not . all isSpace) . lines
  where
    chunks [] = []
    chunks (l:ls) = let (cont, rest) = span (isSpace . headOr ' ') ls
                    in unwords (l : cont) : chunks rest
    headOr d s = if null s then d else head s
    parseDef s = case breakOn " = " s of
      Just (nm, rhs) -> (trim nm, fst (let (e,ts)=parseExpr (tokenize rhs) in apps e ts))
      Nothing        -> error ("parseDump: no '=' in: " ++ s)

breakOn :: String -> String -> Maybe (String, String)
breakOn pat = go ""
  where go acc s@(c:cs) | take (length pat) s == pat = Just (reverse acc, drop (length pat) s)
                        | otherwise = go (c:acc) cs
        go _ [] = Nothing

trim :: String -> String
trim = f . f where f = reverse . dropWhile isSpace

------------------------------------------------------------ reduction

spine :: Tm -> (Tm, [Tm])
spine (Ap f a) = let (h, as) = spine f in (h, as ++ [a])
spine t = (t, [])

rebuild :: Tm -> [Tm] -> Tm
rebuild = foldl' Ap

-- combinator reduction rules: arity and the rewrite on the first `arity` args
rule :: String -> Maybe (Int, [Tm] -> Tm)
rule "S"   = Just (3, \[x,y,z]   -> Ap (Ap x z) (Ap y z))
rule "K"   = Just (2, \[x,_]     -> x)
rule "I"   = Just (1, \[x]       -> x)
rule "B"   = Just (3, \[x,y,z]   -> Ap x (Ap y z))
rule "C"   = Just (3, \[x,y,z]   -> Ap (Ap x z) y)
rule "A"   = Just (2, \[_,y]     -> y)
rule "U"   = Just (2, \[x,y]     -> Ap y x)
rule "Z"   = Just (3, \[x,y,_]   -> Ap x y)
rule "P"   = Just (3, \[x,y,z]   -> Ap (Ap z x) y)
rule "R"   = Just (3, \[x,y,z]   -> Ap (Ap y z) x)
rule "O"   = Just (4, \[x,y,_,w] -> Ap (Ap w x) y)
rule "S'"  = Just (4, \[x,y,z,w] -> Ap (Ap x (Ap y w)) (Ap z w))
rule "B'"  = Just (4, \[x,y,z,w] -> Ap (Ap x y) (Ap z w))
rule "C'"  = Just (4, \[x,y,z,w] -> Ap (Ap x (Ap y w)) z)
rule "C'B" = Just (4, \[x,y,z,w] -> Ap (Ap x z) (Ap y w))
rule "K2"  = Just (3, \(x:_)     -> x)
rule "K3"  = Just (4, \(x:_)     -> x)
rule "K4"  = Just (5, \(x:_)     -> x)
rule "J"   = Just (3, \[x,_,z]   -> Ap z x)
rule "Y"   = Just (1, \[x]       -> Ap x (Ap (Lf "Y") x))
rule _     = Nothing

-- human description of each combinator rule (shown as "what's happening")
ruleDesc :: String -> String
ruleDesc "S"   = "S x y z -> x z (y z)"
ruleDesc "K"   = "K x y -> x"
ruleDesc "I"   = "I x -> x"
ruleDesc "B"   = "B x y z -> x (y z)"
ruleDesc "C"   = "C x y z -> x z y"
ruleDesc "A"   = "A x y -> y"
ruleDesc "U"   = "U x y -> y x"
ruleDesc "Z"   = "Z x y z -> x y"
ruleDesc "P"   = "P x y z -> z x y"
ruleDesc "R"   = "R x y z -> y z x"
ruleDesc "O"   = "O x y z w -> w x y"
ruleDesc "S'"  = "S' x y z w -> x (y w) (z w)"
ruleDesc "B'"  = "B' x y z w -> x y (z w)"
ruleDesc "C'"  = "C' x y z w -> x (y w) z"
ruleDesc "C'B" = "C'B x y z w -> x z (y w)"
ruleDesc "K2"  = "K2 x y z -> x"
ruleDesc "K3"  = "K3 x y z w -> x"
ruleDesc "K4"  = "K4 x y z w v -> x"
ruleDesc "J"   = "J x y z -> z x"
ruleDesc "Y"   = "Y x -> x (Y x)"
ruleDesc c     = c

-- one normal-order step, returning (rule applied, new term): unfold a definition
-- at the head, else fire a saturated combinator at the head, else reduce the
-- leftmost reducible argument.
step :: M.Map String Tm -> Tm -> Maybe (String, Tm)
step defs t =
  case spine t of
    (Lf h, args)
      | Just body <- M.lookup h defs              -> Just ("unfold " ++ unqual h, rebuild body args)
      | Just (n, f) <- rule h, length args >= n   -> Just (ruleDesc h, rebuild (f (take n args)) (drop n args))
    (h, args) -> reduceArg h args
  where
    reduceArg h = go []
      where go _   []     = Nothing
            go acc (a:as) = case step defs a of
              Just (r, a') -> Just (r, rebuild h (reverse acc ++ a' : as))
              Nothing      -> go (a:acc) as

-- the reduction as (rule-that-produced-this, term); the head has no rule.
reduceTrace :: Int -> M.Map String Tm -> Tm -> [(Maybe String, Tm)]
reduceTrace lim defs t0 = (Nothing, t0) : go lim t0
  where go 0 _ = []
        go n t = case step defs t of
          Just (r, t') -> (Just r, t') : go (n - 1) t'
          Nothing      -> []

------------------------------------------------------------ iota expansion (per step)

-- replace the dead `patternMatchFail "..."` branch with I (never evaluated)
unPMF :: Tm -> Tm
unPMF (Ap (Lf p) _) | "patternMatchFail" `isInfixOf` p = Lf "I"
unPMF (Ap a b)                                          = Ap (unPMF a) (unPMF b)
unPMF t                                                 = t

absTm :: String -> Tm -> Tm
absTm x (Lf s) | s == x    = Lf "I"
               | otherwise = Ap (Lf "K") (Lf s)
absTm x (Ap a b)           = Ap (Ap (Lf "S") (absTm x a)) (absTm x b)

-- inline all definitions into a closed combinator term (Y-wrapping self-recursive
-- defs, dropping pattern-match failures)
inlineClosed :: M.Map String Tm -> Tm -> Tm
inlineClosed defs0 = go []
  where
    defs = M.mapWithKey ywrap (M.map unPMF defs0)
    ywrap n b | occurs n b = Ap (Lf "Y") (absTm n b)
              | otherwise  = b
    go st (Ap a b) = Ap (go st a) (go st b)
    go st (Lf s) | s `elem` st               = Lf s
                 | Just t <- M.lookup s defs  = go (s:st) t
                 | otherwise                  = Lf s

algebraDefs :: [(String, String)]
algebraDefs =
  [ ("B","S (K S) K"), ("C","S (B B S) (K K)"), ("A","K I"), ("U","C I")
  , ("Z","B K"), ("P","B C (C I)"), ("R","C C"), ("O","B (B K) (B C (C I))")
  , ("J","B K (C I)"), ("S'","B (B S) B"), ("B'","B B"), ("C'","B (B C) B")
  , ("C'B","C' B"), ("K2","B K K"), ("K3","B K2 K"), ("K4","B K3 K") ]

ySK :: String   -- Curry's Y in S/K/I
ySK = "((S ((S ((S (K S)) ((S (K K)) I))) ((S ((S (K S)) (K I))) (K I)))) ((S ((S (K S)) ((S (K K)) I))) ((S ((S (K S)) (K I))) (K I))))"

parseTmStr :: String -> Tm
parseTmStr s = let (e, ts) = parseExpr (tokenize s) in fst (apps e ts)

isComb :: String -> Bool
isComb s = s `elem` ["S","K","I","Y"] || s `elem` map fst algebraDefs

combSK :: String -> Tm
combSK "S" = Lf "S"
combSK "K" = Lf "K"
combSK "I" = Lf "I"
combSK "Y" = parseTmStr ySK
combSK n   = case lookup n algebraDefs of
  Just d  -> expand (parseTmStr d)
  Nothing -> error ("combSK: " ++ n)
  where expand (Ap a b) = Ap (expand a) (expand b)
        expand (Lf s)   = combSK s

toSK :: Tm -> Either String Tm
toSK (Ap a b) = Ap <$> toSK a <*> toSK b
toSK (Lf s) | isComb s = Right (combSK s) | otherwise = Left s

i1, iI, iK, iS :: Tm
i1 = Lf "1"; iI = Ap i1 i1; iK = foldr1 Ap [i1,i1,i1,i1]; iS = Ap i1 iK

skToIota :: Tm -> Tm
skToIota (Lf "S") = iS
skToIota (Lf "K") = iK
skToIota (Lf "I") = iI
skToIota (Lf s)   = error ("skToIota: " ++ s)
skToIota (Ap a b) = Ap (skToIota a) (skToIota b)

encodeIota :: Tm -> String
encodeIota (Lf _)   = "1"
encodeIota (Ap a b) = '0' : encodeIota a ++ encodeIota b

stepIota :: M.Map String Tm -> Tm -> String
stepIota defs t = case toSK (inlineClosed defs (unPMF t)) of
  Right sk  -> encodeIota (skToIota sk)
  Left bad  -> "IMPURE:" ++ bad

------------------------------------------------------------ display

unqual :: String -> String
unqual s | take 1 s == "\"" = "\"...\""                       -- string literal
         | otherwise        = reverse (takeWhile (/= '.') (reverse s))

-- recognise a Peano numeral  S (S (... Z))  and show it as an integer
numeral :: Tm -> Maybe Int
numeral (Lf z)            | unqual z == "Z" = Just 0
numeral (Ap (Lf s) t)     | unqual s == "S" = (+ 1) <$> numeral t
numeral _                                   = Nothing

showTm :: Tm -> String
showTm = top
  where
    top t | Just k <- numeral t = show k
    top t | isBottom t = "_|_"
    top (Lf s)   = unqual s
    top (Ap f a) = top f ++ " " ++ arg a
    arg t | Just k <- numeral t = show k
    arg t | isBottom t = "_|_"
    arg (Lf s)   = unqual s
    arg t        = "(" ++ top t ++ ")"
    isBottom (Ap (Lf p) _) = "patternMatchFail" `isInfixOf` p   -- dead default branch
    isBottom _             = False

occurs :: String -> Tm -> Bool
occurs x (Lf s)   = s == x
occurs x (Ap a b) = occurs x a || occurs x b

-- the self-recursive definitions (e.g. lt) -- the "interesting" call steps
recDefs :: M.Map String Tm -> [String]
recDefs defs = [ n | (n, b) <- M.toList defs, occurs n b ]

isTraced :: [String] -> Tm -> Bool
isTraced rec t = case fst (spine t) of Lf h -> h `elem` rec; _ -> False

------------------------------------------------------------

main :: IO ()
main = do
  raw <- getArgs
  let full = "--full" `elem` raw
      flags = ["--full", "--iota"]
  (path, root, lim) <- case filter (`notElem` flags) raw of
    [p, r]    -> pure (p, r, 5000)
    [p, r, n] -> pure (p, r, read n)
    _ -> error "usage: reduce [--full|--iota] DUMPFILE ROOTNAME [stepLimit]"
  defs <- parseDump <$> readFile path
  let trace = reduceTrace lim defs (Lf root)
  if "--iota" `elem` raw
    then mapM_ (\(mr, t) -> putStrLn (maybe "" id mr ++ "\t" ++ stepIota defs t)) trace
    else do
      let steps = zip [0 :: Int ..] trace
          n     = length steps - 1
          rec   = recDefs defs
          shown = if full then steps
                  else [ s | s@(i, (_, t)) <- steps, i == 0 || i == n || isTraced rec t ]
      mapM_ (\(i, (mr, t)) -> putStrLn (pad i ++ maybe "" (\r -> "[" ++ r ++ "]  ") mr ++ showTm t)) shown
      putStrLn ("(" ++ show n ++ " combinator steps"
                ++ (if full then "" else "; showing function-call steps -- use --full for all") ++ ")")
  where pad i = let s = show i in replicate (4 - length s) ' ' ++ s ++ "  "
