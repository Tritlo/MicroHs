-- Strictly-less-than on Church numerals, as pure lambda terms.
-- Everything bracket-abstracts to pure combinators (no Int, no primitives),
-- so `lt` renders as a real Barker-Iota program (~15k symbols).
module ChurchLt(zero, suc, isZero, prd, sub, leq, lt) where
import qualified Prelude()

true   = \x y -> x
false  = \x y -> y
zero   = \f x -> x
suc    = \n f x -> f (n f x)
isZero = \n -> n (\u -> false) true
prd    = \n f x -> n (\g h -> h (g f)) (\u -> x) (\u -> u)   -- Kleene predecessor
sub    = \m n -> n prd m                                     -- truncated subtraction
leq    = \m n -> isZero (sub m n)                            -- m <= n
lt     = \m n -> leq (suc m) n                               -- m <  n
