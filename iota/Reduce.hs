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

-- one normal-order step: unfold a definition at the head, else fire a saturated
-- combinator at the head, else reduce the leftmost reducible argument.
step :: M.Map String Tm -> Tm -> Maybe Tm
step defs t =
  case spine t of
    (Lf h, args)
      | Just body <- M.lookup h defs              -> Just (rebuild body args)
      | Just (n, f) <- rule h, length args >= n   -> Just (rebuild (f (take n args)) (drop n args))
    (h, args) -> reduceArg h args
  where
    reduceArg h = go []
      where go _   []     = Nothing
            go acc (a:as) = case step defs a of
              Just a' -> Just (rebuild h (reverse acc ++ a' : as))
              Nothing -> go (a:acc) as

reduceTrace :: Int -> M.Map String Tm -> Tm -> [Tm]
reduceTrace lim defs = go lim
  where go 0 t = [t]
        go n t = t : maybe [] (go (n-1)) (step defs t)

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
  (path, root, lim) <- case filter (/= "--full") raw of
    [p, r]    -> pure (p, r, 5000)
    [p, r, n] -> pure (p, r, read n)
    _ -> error "usage: reduce [--full] DUMPFILE ROOTNAME [stepLimit]"
  defs <- parseDump <$> readFile path
  let steps = zip [0 :: Int ..] (reduceTrace lim defs (Lf root))
      n     = length steps - 1
      rec   = recDefs defs
      shown = if full then steps
              else [ s | s@(i, t) <- steps, i == 0 || i == n || isTraced rec t ]
  mapM_ (\(i, t) -> putStrLn (pad i ++ showTm t)) shown
  putStrLn ("(" ++ show n ++ " combinator steps"
            ++ (if full then "" else "; showing function-call steps -- use --full for all") ++ ")")
  where pad i = let s = show i in replicate (4 - length s) ' ' ++ s ++ "  "
