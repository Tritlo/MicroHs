-- | Semantic validator: parse a Barker-Iota 0/1 string, apply it to free
-- argument names, normal-order reduce, and print the normal form.
-- Reduction rules:  i x = x S K ;  S x y z = x z (y z) ;  K x y = x ;  I x = x.
module Main (main) where

import System.Environment (getArgs)

data E = I_ | Sc | Kc | Ic | Ap E E | Fv String deriving Eq

-- parse prefix iota: '1' -> i, '0' a b -> (a b)
parse :: String -> (E, String)
parse ('1':r) = (I_, r)
parse ('0':r) = let (a, r1) = parse r; (b, r2) = parse r1 in (Ap a b, r2)
parse s       = error ("parse: " ++ take 8 s)

-- one normal-order (leftmost-outermost) reduction step, if any
step :: E -> Maybe E
step (Ap I_ x)                 = Just (Ap (Ap x Sc) Kc)   -- i x = x S K
step (Ap Ic x)                 = Just x
step (Ap (Ap Kc x) _)          = Just x
step (Ap (Ap (Ap Sc x) y) z)   = Just (Ap (Ap x z) (Ap y z))
step (Ap a b) = case step a of
  Just a' -> Just (Ap a' b)
  Nothing -> Ap a <$> step b
step _ = Nothing

reduce :: Int -> E -> E
reduce 0 e = e
reduce n e = maybe e (reduce (n-1)) (step e)

pretty :: E -> String
pretty I_ = "i"; pretty Sc = "S"; pretty Kc = "K"; pretty Ic = "I"
pretty (Fv s) = s
pretty (Ap a b) = "(" ++ pretty a ++ " " ++ pretty b ++ ")"

main :: IO ()
main = do
  (code:args) <- getArgs
  let (e, _) = parse code
      applied = foldl (\acc a -> Ap acc (Fv a)) e args
  putStrLn (pretty (reduce 1000000 applied))
