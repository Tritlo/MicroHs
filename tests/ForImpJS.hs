module ForImpJS where
import Data.ByteString(ByteString)
import qualified Data.ByteString.Char8 as BC
import Foreign.C.String(CString, withCAString, peekCAString)
import Mhs.JavaScript(JSVal)

foreign import javascript "console.log('log: ' + UTF8ToString($0))" clog :: CString -> IO ()
foreign import javascript "return stringToNewUTF8('PRE' + UTF8ToString($0))" pre :: CString -> IO CString
foreign import javascript "return $0 + $1"          add   :: Int -> Int -> Int
foreign import javascript "return $0 * $1"          mul   :: Double -> Double -> Double
foreign import javascript "return $0 < $1"          lt    :: Int -> Int -> Bool
foreign import javascript "return !$0"              nt    :: Bool -> Bool
foreign import javascript "return $0 === 3000000000" isBig :: Word -> Bool
foreign import javascript "return $0 + 1"           nextc :: Char -> Char
foreign import javascript "return $0 + '!'"         bang  :: ByteString -> IO ByteString
foreign import javascript "return { n: $0 }"        mkObj :: Int -> IO JSVal
foreign import javascript "return $0.n * 2"         getN  :: JSVal -> IO Int
foreign import javascript "wrapper"                 mkCB  :: (Int -> Int -> IO Int) -> IO JSVal
foreign import javascript "return $0(20, 3)"        callCB :: JSVal -> IO Int

hlog :: String -> IO ()
hlog s = withCAString s clog

main :: IO ()
main = do
  hlog "JS log"
  hlog "JS log again"
  hlog $ show $ add 3 4
  hlog $ show $ mul 3 4
  s <- withCAString "-test" $ \ p -> pre p >>= peekCAString
  putStrLn s
  print (lt 1 2, lt 2 1, nt False)
  print (isBig 3000000000, isBig 17)
  print (nextc 'a')
  bs <- bang (BC.pack "boom")
  putStrLn (BC.unpack bs)
  o <- mkObj 21
  getN o >>= print
  cb <- mkCB (\ x y -> return (x - y))
  callCB cb >>= print
