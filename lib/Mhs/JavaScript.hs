-- | A GC-managed reference to a JavaScript value, modelled on GHC's wasm @JSVal@.
--
-- A 'JSVal' is a handle into the runtime's JS object registry, wrapped in a
-- 'ForeignPtr' whose finalizer frees the slot when the 'JSVal' is collected.
-- A @foreign import javascript@ takes and returns 'JSVal' directly; the type
-- is opaque and not serializable.
module Mhs.JavaScript(JSVal) where
import qualified Prelude(); import MiniPrelude
import Foreign.ForeignPtr(ForeignPtr)

-- The phantom argument distinguishes a JS-object handle from an ordinary
-- foreign pointer; the name must agree with jsScalarTag in ExpPrint.
data JSValRep

-- | A garbage-collected reference to a JavaScript value.
newtype JSVal = JSVal (ForeignPtr JSValRep)
