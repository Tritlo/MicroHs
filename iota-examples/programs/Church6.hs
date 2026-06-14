-- Church numeral 6: apply f six times.  Compiles to five nested (S B)
-- "successor" gadgets around an I, i.e. succ^5 of Church-1.
module Church6(six) where
import qualified Prelude()

six f x = f (f (f (f (f (f x)))))
