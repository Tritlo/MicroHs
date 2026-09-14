module DirectWasm(main) where

import Data.Int(Int64)
import Data.Word(Word64)

foreign import wasm "mhs_example add_i32" addInt :: Int -> Int -> Int
foreign import wasm "mhs_example add_u32" addWord :: Word -> Word -> Word
foreign import wasm "mhs_example add_i64" addInt64 :: Int64 -> Int64 -> Int64
foreign import wasm "mhs_example add_u64" addWord64 :: Word64 -> Word64 -> Word64
foreign import wasm "mhs_example add_f32" addFloat :: Float -> Float -> Float
foreign import wasm "mhs_example add_f64" addDouble :: Double -> Double -> Double
foreign import wasm "mhs_example set_value" setValue :: Int -> IO ()
foreign import wasm "mhs_example get_value" getValue :: IO Int

-- | Call the scalar exports of direct.wat.
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
