module DirectJavascript(main) where

import Data.Int(Int64)
import Data.Word(Word64)

foreign import javascript "return $0 + $1" addInt :: Int -> Int -> Int
foreign import javascript "return $0 + $1" addWord :: Word -> Word -> Word
foreign import javascript "return $0 + $1" addInt64 :: Int64 -> Int64 -> Int64
foreign import javascript "return $0 + $1" addWord64 :: Word64 -> Word64 -> Word64
foreign import javascript "return $0 + $1" addFloat :: Float -> Float -> Float
foreign import javascript "return $0 + $1" addDouble :: Double -> Double -> Double
foreign import javascript "globalThis.mhsWasmExampleValue = $0" setValue :: Int -> IO ()
foreign import javascript "return globalThis.mhsWasmExampleValue" getValue :: IO Int
foreign import javascript "return $0 > 0 ? 1 : 0" positiveWord :: Word -> Int
foreign import javascript "return $0 > 0n ? 1 : 0" positiveWord64 :: Word64 -> Int

-- | Call JavaScript with number and bigint values through typed WASM imports.
main :: IO ()
main = do
  print $ addInt 20 22
  print $ addWord 4294967295 1
  print $ addInt64 9007199254740993 10
  print $ addWord64 18446744073709551615 1
  print $ addFloat 1.25 2.5
  print $ addDouble 1.25 2.5
  setValue 123
  print =<< getValue
  print $ positiveWord 4294967295
  print $ positiveWord64 18446744073709551615
