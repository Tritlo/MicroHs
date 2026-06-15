module Lt(test) where
import qualified Prelude(); import Data.Typeable
data Bool = F | T
data Nat  = Z | S Nat
lt :: Nat -> Nat -> Bool
lt Z     (S _) = T
lt _     Z     = F
lt (S m) (S n) = lt m n
test :: Bool
test = lt (S (S Z)) (S (S (S Z)))   -- 2 < 3
