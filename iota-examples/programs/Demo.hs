-- Basic combinators: each definition is a tiny lambda term that MicroHs
-- bracket-abstracts to a closed combinator term (no Prelude, no primitives).
module Demo(comp, tw, sk, idf) where
import qualified Prelude()

comp f g x = f (g x)   -- B
tw   f x   = f (f x)   -- S B I   (Church numeral 2)
sk   x y   = x         -- K
idf  x     = x         -- I
