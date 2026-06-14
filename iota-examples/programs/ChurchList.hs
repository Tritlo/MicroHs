-- Functional list encodings, all pure combinators.
--   Church (right-fold) lists: a list *is* its foldr.   foldr_ = P, app = S' B
--   Difference lists layered on top:                    dnil = I, dapp = B
-- `l3` is the Church list [a,b,c] (a,b,c become combinator plumbing).
module ChurchList(nil, cons, foldr_, app, dnil, dsing, dapp, toL, l3) where
import qualified Prelude()

nil    = \c n -> n
cons   = \h t c n -> c h (t c n)
foldr_ = \f z xs -> xs f z
app    = \xs ys c n -> xs c (ys c n)        -- O(1) append on fold-lists

dnil   = \k -> k                            -- difference list: identity
dsing  = \h k -> cons h k
dapp   = \a b k -> a (b k)                  -- difference-list append = compose
toL    = \d -> d nil

l3     = \a b c -> cons a (cons b (cons c nil))   -- the list [a,b,c]
