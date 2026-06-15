-- | Identity-tracking graph reducer.  Reduces a -ddump-combinator definition in
-- normal order, but every node carries a stable ID so an animator can tell which
-- subtrees persist, which are copied (by S, S', Y ...), and which are dropped (by
-- K, A ...).  Emits, per step:  RULE \t  id-tagged s-expression of the term.
--
-- s-expr:  (ID @ LEFT RIGHT)  for an application,  (ID LABEL)  for a leaf.
module Main (main) where

import Data.Char (isSpace)
import Data.List (isInfixOf, foldl')
import qualified Data.Map.Strict as M
import System.Environment (getArgs)

------------------------------------------------------------ a tiny state monad (fresh ids)
newtype F a = F { runF :: Int -> (a, Int) }
instance Functor F where fmap f (F g) = F (\s -> let (a, s') = g s in (f a, s'))
instance Applicative F where
  pure a = F (\s -> (a, s))
  F f <*> F a = F (\s -> let (g, s') = f s; (x, s'') = a s' in (g x, s''))
instance Monad F where F a >>= k = F (\s -> let (x, s') = a s in runF (k x) s')

fresh :: F Int
fresh = F (\s -> (s, s + 1))

------------------------------------------------------------ terms
data Tm = TAp Tm Tm | TLf String                 -- plain (from the parser)
data G  = GAp Int G G | GLf Int String           -- id-tagged

------------------------------------------------------------ parsing (as in Reduce.hs)
data Tok = LP | RP | Atom String

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
parseExpr (Atom a : ts) = (TLf a, ts)
parseExpr (LP : ts)     =
  let (e, ts') = parseExpr ts; (e', ts'') = apps e ts'
  in case ts'' of (RP : r) -> (e', r); _ -> error "parseExpr: expected )"
parseExpr _ = error "parseExpr"

apps :: Tm -> [Tok] -> (Tm, [Tok])
apps acc ts@(RP:_) = (acc, ts)
apps acc []        = (acc, [])
apps acc ts        = let (e, ts') = parseExpr ts in apps (TAp acc e) ts'

parseDump :: String -> M.Map String Tm
parseDump = M.fromList . map def . chunks . filter (not . all isSpace) . lines
  where
    chunks [] = []
    chunks (l:ls) = let (cont, rest) = span (isSpace . headOr ' ') ls in unwords (l:cont) : chunks rest
    headOr d s = if null s then d else head s
    def s = case breakOn " = " s of
      Just (nm, rhs) -> (trim nm, fst (let (e,ts)=parseExpr (tokenize rhs) in apps e ts))
      Nothing        -> error ("parseDump: " ++ s)

breakOn :: String -> String -> Maybe (String, String)
breakOn pat = go "" where
  go acc s@(c:cs) | take (length pat) s == pat = Just (reverse acc, drop (length pat) s)
                  | otherwise = go (c:acc) cs
  go _ [] = Nothing

trim :: String -> String
trim = f . f where f = reverse . dropWhile isSpace

unqual :: String -> String
unqual s | take 1 s == "\"" = "_|_"
         | otherwise        = reverse (takeWhile (/= '.') (reverse s))

-- drop the dead `patternMatchFail "..."` branch -> a single _|_ leaf
unPMF :: Tm -> Tm
unPMF (TAp (TLf p) _) | "patternMatchFail" `isInfixOf` p = TLf "_|_"
unPMF (TAp a b) = TAp (unPMF a) (unPMF b)
unPMF t = t

------------------------------------------------------------ id tagging
tag :: Tm -> F G
tag (TLf s)   = (\i -> GLf i s) <$> fresh
tag (TAp a b) = do a' <- tag a; b' <- tag b; i <- fresh; pure (GAp i a' b')

copyG :: G -> F G                                -- deep copy with fresh ids
copyG (GLf _ s)   = (\i -> GLf i s) <$> fresh
copyG (GAp _ a b) = do a' <- copyG a; b' <- copyG b; i <- fresh; pure (GAp i a' b')

------------------------------------------------------------ reduction
spineIds :: G -> (G, [(Int, G)])                 -- (head, [(appNodeId, arg)])
spineIds (GAp i f a) = let (h, ps) = spineIds f in (h, ps ++ [(i, a)])
spineIds t = (t, [])

ap2 :: G -> G -> F G
ap2 l r = (\i -> GAp i l r) <$> fresh

-- | Per-step provenance for the animator: which argument subtrees this rule
-- *discards* (the branch not selected), which it *copies* (a duplicated argument,
-- as (original-root, copy-root) pairs), and the *redex* head that fired.  Ids are
-- iota-level (see 'iroot'), so they line up with the rendered ι-tree.
data Prov = Prov { pDrop :: [Int], pCopy :: [(Int, Int)], pRedex :: Maybe Int }

emptyProv :: Prov
emptyProv = Prov [] [] Nothing

-- Iota-level root id of a combinator subtree: an application keeps its id under
-- expandIota; a leaf becomes its gadget, whose root id is gid 0 (see 'tagGadget').
iroot :: G -> Int
iroot (GAp i _ _) = i
iroot (GLf i _)   = negate (i*100000 + 1)

-- combinator rules: each yields the redex result (fresh ids for new app nodes,
-- deep copies for duplicated arguments) together with what it dropped / copied.
gRule :: String -> Maybe (Int, [G] -> F (G, Prov))
gRule "S"   = Just (3, \[x,y,z]   -> do z2 <- copyG z; a <- ap2 x z; b <- ap2 y z2; r <- ap2 a b; pure (r, copy z z2))
gRule "K"   = Just (2, \[x,y]     -> pure (x, drop1 y))
gRule "I"   = Just (1, \[x]       -> pure (x, none))
gRule "B"   = Just (3, \[x,y,z]   -> do yz <- ap2 y z; r <- ap2 x yz; pure (r, none))
gRule "C"   = Just (3, \[x,y,z]   -> do xz <- ap2 x z; r <- ap2 xz y; pure (r, none))
gRule "A"   = Just (2, \[x,y]     -> pure (y, drop1 x))
gRule "U"   = Just (2, \[x,y]     -> do r <- ap2 y x; pure (r, none))
gRule "Z"   = Just (3, \[x,y,z]   -> do r <- ap2 x y; pure (r, drop1 z))
gRule "P"   = Just (3, \[x,y,z]   -> do zx <- ap2 z x; r <- ap2 zx y; pure (r, none))
gRule "R"   = Just (3, \[x,y,z]   -> do yz <- ap2 y z; r <- ap2 yz x; pure (r, none))
gRule "O"   = Just (4, \[x,y,z,w] -> do wx <- ap2 w x; r <- ap2 wx y; pure (r, drop1 z))
gRule "S'"  = Just (4, \[x,y,z,w] -> do w2 <- copyG w; yw <- ap2 y w; zw <- ap2 z w2; a <- ap2 x yw; r <- ap2 a zw; pure (r, copy w w2))
gRule "B'"  = Just (4, \[x,y,z,w] -> do zw <- ap2 z w; xy <- ap2 x y; r <- ap2 xy zw; pure (r, none))
gRule "C'"  = Just (4, \[x,y,z,w] -> do yw <- ap2 y w; a <- ap2 x yw; r <- ap2 a z; pure (r, none))
gRule "C'B" = Just (4, \[x,y,z,w] -> do yw <- ap2 y w; xz <- ap2 x z; r <- ap2 xz yw; pure (r, none))
gRule "K2"  = Just (3, \[x,y,z]   -> pure (x, drops [y,z]))
gRule "K3"  = Just (4, \[x,y,z,w] -> pure (x, drops [y,z,w]))
gRule "K4"  = Just (5, \[x,y,z,w,v] -> pure (x, drops [y,z,w,v]))
gRule "J"   = Just (3, \[x,y,z]   -> do r <- ap2 z x; pure (r, drop1 y))
gRule "Y"   = Just (1, \[x]       -> do x2 <- copyG x; yn <- (\i -> GLf i "Y") <$> fresh; yx <- ap2 yn x2; r <- ap2 x yx; pure (r, copy x x2))
gRule _     = Nothing

none :: Prov
none = emptyProv
drop1 :: G -> Prov
drop1 g = emptyProv { pDrop = [iroot g] }
drops :: [G] -> Prov
drops gs = emptyProv { pDrop = map iroot gs }
copy :: G -> G -> Prov
copy orig cp = emptyProv { pCopy = [(iroot orig, iroot cp)] }

ruleDesc :: String -> String
ruleDesc c = case lookup c tbl of Just d -> d; Nothing -> c
  where tbl = [ ("S","S x y z -> x z (y z)"), ("K","K x y -> x"), ("I","I x -> x")
              , ("B","B x y z -> x (y z)"), ("C","C x y z -> x z y"), ("A","A x y -> y")
              , ("U","U x y -> y x"), ("Z","Z x y z -> x y"), ("P","P x y z -> z x y")
              , ("R","R x y z -> y z x"), ("O","O x y z w -> w x y")
              , ("S'","S' x y z w -> x (y w) (z w)"), ("B'","B' x y z w -> x y (z w)")
              , ("C'","C' x y z w -> x (y w) z"), ("C'B","C'B x y z w -> x z (y w)")
              , ("K2","K2 x y z -> x"), ("K3","K3 x y z w -> x"), ("K4","K4 x y z w v -> x")
              , ("J","J x y z -> z x"), ("Y","Y x -> x (Y x)") ]

reattach :: G -> [(Int, G)] -> G                 -- rebuild the upper spine, keeping app ids
reattach = foldl' (\acc (i, arg) -> GAp i acc arg)

-- When the fixpoint combinator unrolls (Y g -> g (Y g)) the next two spine args
-- are lt's numeral arguments; surface them so the animator can mark the call.
note :: String -> [(Int, G)] -> String
note "Y" ((_,a):(_,b):_) | Just i <- numG a, Just j <- numG b = "   lt " ++ show i ++ " " ++ show j
note _ _                                                      = ""

-- one normal-order step: (rule description, provenance, new whole term).  The term
-- is already closed (no named defs), so there is nothing to unfold: every step is
-- a real combinator rule, and we record what it dropped / copied for the animator.
gStep :: G -> Maybe (F (String, Prov, G))
gStep g =
  case spineIds g of
    (GLf hid name, ps)
      | Just (n, f) <- gRule name, length ps >= n ->
          Just $ do let (used, rest) = splitAt n ps
                    (res, prov) <- f (map snd used)
                    pure ( ruleDesc name ++ note name rest
                         , prov { pRedex = Just (iroot (GLf hid name)) }
                         , reattach res rest )
    (h, ps) -> argStep h ps
  where
    argStep h = go []
      where go _ [] = Nothing
            go done ((i,arg):rest) = case gStep arg of
              Just act -> Just $ do (r, prov, arg') <- act
                                    pure (r, prov, reattach h (reverse done ++ (i,arg') : rest))
              Nothing  -> go ((i,arg):done) rest

greduce :: Int -> G -> F [(Maybe String, Maybe Prov, G)]
greduce lim g0 = ((Nothing, Nothing, g0) :) <$> loop lim g0
  where loop 0 _ = pure []
        loop n g = case gStep g of
          Nothing  -> pure []
          Just act -> do (r, prov, g') <- act; rest <- loop (n-1) g'; pure ((Just r, Just prov, g') : rest)

------------------------------------------------------------ output
sexp :: G -> String
sexp (GLf i s)   = "(" ++ show i ++ " " ++ unqual s ++ ")"
sexp (GAp i a b) = "(" ++ show i ++ " @ " ++ sexp a ++ " " ++ sexp b ++ ")"

-- Scott numerals as combinators (the constructors are inlined: Lt.Z = K, Lt.S = J),
-- so a number n is the tower J^n K.  numG reads one back out (K = 0, J t = 1+t);
-- numTop only fires on a J-headed tower so a bare K combinator isn't shown as "0".
numG :: G -> Maybe Int
numG (GLf _ s)            | unqual s == "K" = Just 0
numG (GAp _ (GLf _ s) t)  | unqual s == "J" = (+ 1) <$> numG t
numG _                                      = Nothing

numTop :: G -> Maybe Int
numTop g@(GAp _ (GLf _ s) _) | unqual s == "J" = numG g
numTop _                                       = Nothing

showGTerm :: G -> String
showGTerm = top
  where
    top g | Just k <- numTop g = show k
    top (GLf _ s)   = unqual s
    top (GAp _ f a) = top f ++ " " ++ arg a
    arg g | Just k <- numTop g = show k
    arg (GLf _ s)   = unqual s
    arg g           = "(" ++ top g ++ ")"

------------------------------------------------------------ iota expansion (derived ids)
-- Each combinator/def leaf becomes its ι-gadget; gadget node ids are derived from
-- the combinator leaf's id (base*K + preorder index), so the same node's gadget
-- keeps its identity across steps.  App-node ids are preserved as-is.
occurs :: String -> Tm -> Bool
occurs x (TLf s)   = s == x
occurs x (TAp a b) = occurs x a || occurs x b

-- Optimised bracket abstraction (the classic K/I/eta/S rules).  Because the
-- recursion variable occurs just once, the K and eta rules keep \x.body close to
-- the size of body itself instead of tripling it the way naive S/K abstraction
-- would -- which keeps the reduction (and the rendered ι-trees) tractable.
absTm :: String -> Tm -> Tm
absTm x t
  | not (occurs x t)                            = TAp (TLf "K") t      -- K: x unused
absTm _ (TLf _)                                 = TLf "I"             -- I: t is x
absTm x (TAp a (TLf y)) | y == x, not (occurs x a) = a                -- eta: \x. a x = a
absTm x (TAp a b)                               = TAp (TAp (TLf "S") (absTm x a)) (absTm x b)

inlineClosed :: M.Map String Tm -> Tm -> Tm
inlineClosed defs0 = go []
  where
    defs = M.mapWithKey yw defs0
    yw n b = if occurs n b then TAp (TLf "Y") (absTm n b) else b
    go st (TAp a b) = TAp (go st a) (go st b)
    go st (TLf s) | s `elem` st                = TLf s
                  | Just t <- M.lookup s defs   = go (s:st) t
                  | otherwise                   = TLf s

algebraDefs :: [(String, String)]
algebraDefs =
  [ ("B","S (K S) K"), ("C","S (B B S) (K K)"), ("A","K I"), ("U","C I")
  , ("Z","B K"), ("P","B C (C I)"), ("R","C C"), ("O","B (B K) (B C (C I))")
  , ("J","B K (C I)"), ("S'","B (B S) B"), ("B'","B B"), ("C'","B (B C) B")
  , ("C'B","C' B"), ("K2","B K K"), ("K3","B K2 K"), ("K4","B K3 K") ]

ySK :: String
ySK = "((S ((S ((S (K S)) ((S (K K)) I))) ((S ((S (K S)) (K I))) (K I)))) ((S ((S (K S)) ((S (K K)) I))) ((S ((S (K S)) (K I))) (K I))))"

parseTmStr :: String -> Tm
parseTmStr s = fst (let (e,ts) = parseExpr (tokenize s) in apps e ts)

isComb :: String -> Bool
isComb s = s `elem` ["S","K","I","Y"] || s `elem` map fst algebraDefs

combSK :: String -> Tm
combSK "S" = TLf "S"
combSK "K" = TLf "K"
combSK "I" = TLf "I"
combSK "Y" = parseTmStr ySK
combSK n   = case lookup n algebraDefs of
  Just d  -> ex (parseTmStr d)
  Nothing -> error ("combSK: " ++ n)
  where ex (TAp a b) = TAp (ex a) (ex b)
        ex (TLf s)   = combSK s

toSK :: Tm -> Either String Tm
toSK (TAp a b) = TAp <$> toSK a <*> toSK b
toSK (TLf s) | s == "_|_" = Right (combSK "I")   -- dead branch -> identity
             | isComb s   = Right (combSK s)
             | otherwise  = Left s

i1, iI, iK, iS :: Tm
i1 = TLf "1"; iI = TAp i1 i1; iK = foldr1 TAp [i1,i1,i1,i1]; iS = TAp i1 iK

skToIota :: Tm -> Tm
skToIota (TLf "S") = iS
skToIota (TLf "K") = iK
skToIota (TLf "I") = iI
skToIota (TLf s)   = error ("skToIota: " ++ s)
skToIota (TAp a b) = TAp (skToIota a) (skToIota b)

leafIota :: M.Map String Tm -> String -> Tm
leafIota defs c
  | c == "_|_" = skToIota (combSK "I")
  | otherwise  = case toSK (inlineClosed defs (TLf c)) of
                   Right sk -> skToIota sk
                   Left _   -> skToIota (combSK "I")

-- Gadget node ids are derived from the combinator leaf's id (base) so the same
-- node's ι-gadget keeps its identity across steps.  They live in a *negative*
-- id space, disjoint from the (non-negative) application-node ids of the term --
-- otherwise, once the fresh counter passes the 100000 stride, a raw app id would
-- collide with some base*100000 region, fusing two nodes (a cycle, an infinite loop).
tagGadget :: Int -> Tm -> G
tagGadget base t = fst (go t 0)
  where gid k = negate (base*100000 + k + 1)
        go (TLf s)   k = (GLf (gid k) s, k+1)
        go (TAp a b) k = let (a',k1) = go a (k+1); (b',k2) = go b k1 in (GAp (gid k) a' b', k2)

expandIota :: M.Map String Tm -> G -> G
expandIota defs (GAp i a b) = GAp i (expandIota defs a) (expandIota defs b)
expandIota defs (GLf i c)   = tagGadget i (leafIota defs c)

main :: IO ()
main = do
  raw <- getArgs
  let iotaMode = "--iota" `elem` raw
  (path, root, lim) <- case filter (/= "--iota") raw of
    [p, r]    -> pure (p, r, 5000)
    [p, r, n] -> pure (p, r, read n)
    _ -> error "usage: morph [--iota] DUMPFILE ROOTNAME [stepLimit]"
  defsTm <- M.map unPMF . parseDump <$> readFile path
  -- Inline every non-recursive definition (the data constructors and the entry
  -- point) and tie the one recursive definition with Y (as iota/Iota.hs does),
  -- giving a *closed* term over primitive combinators.  Reducing it with no named
  -- defs means there is nothing to "unfold": every step is a real combinator/iota
  -- rule, and the recursion shows up honestly as Y x -> x (Y x).
  let root0  = inlineClosed defsTm (TLf root)
      action = do g0 <- tag root0; greduce lim g0
      (trace, _) = runF action 0
      line mr mprov g
        | iotaMode  = maybe "" id mr ++ "\t" ++ showGTerm g ++ "\t" ++ sexp (expandIota M.empty g)
                                      ++ "\t" ++ maybe "" provStr mprov
        | otherwise = maybe "" id mr ++ "\t" ++ sexp g
  mapM_ (\(mr, mprov, g) -> putStrLn (line mr mprov g)) trace

-- provenance, packed onto the 4th field: redex head, discarded roots, copy pairs.
provStr :: Prov -> String
provStr (Prov ds cs r) = unwords $
     [ "R:" ++ show i        | Just i <- [r] ]
  ++ [ "D:" ++ show d        | d <- ds ]
  ++ [ "C:" ++ show s ++ ":" ++ show t | (s,t) <- cs ]
