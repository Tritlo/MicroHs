-- Primitives have no combinatory form, so they render as opaque leaf BOXES
-- (e.g. <+>, <IO.return>) and suppress the iota encoding for that term.
-- `mix` shows a primitive sitting amongst pure combinators.
module Prim(addp, retp, mix) where
import qualified Prelude()
import Primitives(primIntAdd, primReturn)

addp    = primIntAdd                  -- the (+) primitive, bare
retp    = primReturn                  -- the IO.return primitive
mix f x = f primIntAdd (f x x)        -- combinators wrapped around a primitive
