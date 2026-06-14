-- Quicksort over Peano naturals, fully primitive-free.
--
-- Using `data` types (not hand-rolled Church/Scott lambdas) keeps Hindley-Milner
-- happy, and MicroHs still Scott-encodes them to pure combinators
-- (Z=K, S=J, Nil=K, Cons=O, F=K, T=A) -- so the whole thing renders as pure iota.
--
-- MicroHs leaves top-level recursion as self-referential defs (quicksort = ...quicksort...);
-- the iota renderer rewrites each into Y (\f. ...f...) to get a finite term.
module Quicksort(qs3, ex312) where
import qualified Prelude(); import Data.Typeable

data Bool = F | T
data Nat  = Z | S Nat
data List = Nil | Cons Nat List

le :: Nat -> Nat -> Bool          -- m <= n
le Z _ = T
le (S _) Z = F
le (S m) (S n) = le m n

leP :: Nat -> List -> List        -- keep elements <= p
leP _ Nil = Nil
leP p (Cons x xs) = case le x p of { T -> Cons x (leP p xs); F -> leP p xs }

gtP :: Nat -> List -> List        -- keep elements > p
gtP _ Nil = Nil
gtP p (Cons x xs) = case le x p of { T -> gtP p xs; F -> Cons x (gtP p xs) }

app :: List -> List -> List
app Nil ys = ys
app (Cons x xs) ys = Cons x (app xs ys)

quicksort :: List -> List
quicksort Nil = Nil
quicksort (Cons p xs) = app (quicksort (leP p xs)) (Cons p (quicksort (gtP p xs)))

qs3 :: Nat -> Nat -> Nat -> List
qs3 a b c = quicksort (Cons a (Cons b (Cons c Nil)))                       -- quicksort [a,b,c]

ex312 :: List
ex312 = quicksort (Cons (S (S (S Z))) (Cons (S Z) (Cons (S (S Z)) Nil)))   -- quicksort [3,1,2]
