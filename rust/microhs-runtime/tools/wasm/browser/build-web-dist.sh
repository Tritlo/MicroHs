#!/usr/bin/env bash
set -euo pipefail

# Build the plain file-tree browser distribution consumed by Combinate CI.
#
# The prewarmed cache is generated at dist time from the current compiler comb
# and library sources, then copied into the dist tree. It is intentionally not
# committed, so it cannot go stale independently of generated/mhs.comb or lib/.
#
# Includes: lib/ only. A browser compile probe for a user module importing
# Prelude and Data.List succeeds with only /lib in the VFS; mhs/ and src/ are
# compiler implementation sources already baked into generated/mhs.comb.

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../../../.." && pwd)"
dist_arg="${1:-${DIST:-$here/dist}}"
if [[ "$dist_arg" = /* ]]; then
  dist="$dist_arg"
else
  dist="$repo/$dist_arg"
fi

case "$dist" in
  ""|"/")
    echo "refusing unsafe dist path: $dist" >&2
    exit 1
    ;;
esac

runtime_wasm="$repo/target/wasm32-unknown-unknown/release/microhs_runtime.wasm"
bench="$repo/target/release/mhs-rust-bench"
comb="$repo/generated/mhs.comb"
warm_modules=(Prelude Data.List Data.Maybe Data.Either Data.Tuple Data.Bool Data.Char)

echo "building embedded browser wasm"
"$here/build-browser-bench.sh"

echo "building native driver"
cargo build \
  --release \
  --manifest-path "$repo/rust/microhs-runtime/Cargo.toml" \
  --bin mhs-rust-bench \
  --quiet

rm -rf "$dist"
mkdir -p "$dist/include/lib"

cp "$runtime_wasm" "$dist/microhs_runtime.wasm"
cp "$here/compiler.mjs" "$dist/compiler.mjs"
cp "$here/host.mjs" "$dist/host.mjs"
cp "$comb" "$dist/mhs.comb"
cp -a "$repo/lib/." "$dist/include/lib/"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cat > "$tmp/Warm.hs" <<'WARM'
module Warm where
import Prelude
import Data.List
import Data.Maybe
import Data.Either
import Data.Tuple
import Data.Bool
import Data.Char

foreign export javascript "warm" warm :: Int -> Int

warm :: Int -> Int
warm n = length (take n (map (+ 1) [1,2,3,4])) + maybe 0 id (Just n) + either id id (Right n) + bool 0 1 (isDigit '7')
WARM

echo "generating prewarm cache"
if ! (
  cd "$tmp"
  "$bench" \
    --input "$comb" \
    --mode main \
    --warmup-iters 0 \
    --iters 1 \
    -- \
    mhs \
    -q \
    -i \
    "-i$tmp" \
    "-i$repo/lib" \
    --no-main \
    -CW \
    "-ddump-combinator-out=$tmp/Warm.dump" \
    Warm
) >"$tmp/prewarm.log" 2>&1; then
  cat "$tmp/prewarm.log" >&2
  exit 1
fi

if [[ ! -s "$tmp/.mhscache" ]]; then
  echo "prewarm cache was not generated" >&2
  exit 1
fi

cp "$tmp/.mhscache" "$dist/prewarm.mhscache"
cache_size="$(wc -c < "$dist/prewarm.mhscache" | tr -d ' ')"
echo "prewarmed cache: $cache_size bytes (${warm_modules[*]})"

DIST="$dist" PREWARM_MODULES="${warm_modules[*]}" node --input-type=module <<'NODE'
import { readdir, writeFile } from "node:fs/promises";
import path from "node:path";

const dist = process.env.DIST;
const includeRoot = path.join(dist, "include");
const includeFiles = {};

async function walk(dir) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      await walk(file);
    } else if (entry.isFile()) {
      const distRel = path.relative(dist, file).split(path.sep).join("/");
      const includeRel = path.relative(includeRoot, file).split(path.sep).join("/");
      includeFiles[distRel] = `/${includeRel}`;
    }
  }
}

await walk(includeRoot);

const manifest = {
  includeFiles: Object.fromEntries(Object.entries(includeFiles).sort()),
  prewarmCache: {
    dist: "prewarm.mhscache",
    vfs: "/.mhscache",
    generatedBy: "build-web-dist.sh",
    warmedModules: process.env.PREWARM_MODULES.split(" "),
  },
};

await writeFile(path.join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
NODE

echo "running dist-only smoke"
DIST="$dist" node --input-type=module <<'NODE'
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const dist = process.env.DIST;
const manifest = JSON.parse(await readFile(path.join(dist, "manifest.json"), "utf8"));
const { createCompiler } = await import(pathToFileURL(path.join(dist, "compiler.mjs")).href);
const files = {};

for (const [distRel, vfsPath] of Object.entries(manifest.includeFiles)) {
  files[vfsPath] = await readFile(path.join(dist, distRel));
}

const prewarm = await readFile(path.join(dist, manifest.prewarmCache.dist));
if (prewarm.length === 0) {
  throw new Error("prewarm cache is empty");
}

const compiler = await createCompiler({
  wasm: await readFile(path.join(dist, "microhs_runtime.wasm")),
  comb: await readFile(path.join(dist, "mhs.comb")),
  files,
});

try {
  const source = `module DistSmoke where
import Prelude
import Data.List

foreign export javascript "distSmoke" distSmoke :: Int -> Int

distSmoke :: Int -> Int
distSmoke n = length (take n [1,2,3,4])
`;

  const out = compiler.compile(source, {
    module: "DistSmoke",
    flags: ["-q", "--no-main"],
  });
  if (!out.dump || out.dump.length === 0) {
    throw new Error(`dist smoke produced empty dump; status=${out.status}; error=${out.error}`);
  }
  const cacheInfo = await stat(path.join(dist, manifest.prewarmCache.dist));
  console.log(`smoke: status=${out.status} dump_bytes=${out.dump.length} prewarm_bytes=${cacheInfo.size}`);
} finally {
  compiler.close();
}
NODE

echo "dist manifest:"
find "$dist" -type f -printf '%P\t%s bytes\n' | sort
