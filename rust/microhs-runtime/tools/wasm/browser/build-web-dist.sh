#!/usr/bin/env bash
set -euo pipefail

# Build the plain file-tree browser distribution consumed by Combinate CI.
#
# Ships the committed generated/base.pkg (the pre-typechecked base library) and
# preloads it via -p, so library modules (Prelude, Data.*, ...) load from the
# package instead of being recompiled from source. This replaces the old prewarm
# .mhscache / -CR path (and its native<->wasm cache-portability question).
# base.pkg is a tracked generated/ artifact kept in sync with generated/mhs.comb;
# the dist smoke compiles through it, so a drift between the two fails the build.
#
# Includes: lib/ source as a fallback for any module not in base.pkg; mhs/ and
# src/ are compiler sources already baked into generated/mhs.comb.

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../../../.." && pwd)"
default_dist="$here/dist"
dist_arg="${1:-${DIST:-$default_dist}}"
if [[ "$dist_arg" = /* ]]; then
  requested_dist="$dist_arg"
else
  requested_dist="$repo/$dist_arg"
fi

dist_parent_arg="$(dirname "$requested_dist")"
dist_name="$(basename "$requested_dist")"
[[ "$dist_name" != "." && "$dist_name" != ".." ]] || {
  echo "refusing unsafe dist path: $requested_dist" >&2
  exit 1
}
mkdir -p "$dist_parent_arg"
dist_parent="$(cd "$dist_parent_arg" && pwd -P)"
dist="$dist_parent/$dist_name"
user_home="$(cd && pwd -P)"

if [[ "$dist" = "/" || "$dist" = "$repo" || "$dist" = "$here" || "$dist" = "$user_home" ]] ||
   [[ "$repo/" = "$dist/"* ]]; then
  echo "refusing unsafe dist path: $dist" >&2
  exit 1
fi

owner_marker=".microhs-web-dist"
if [[ -e "$dist" && "$dist" != "$default_dist" && ! -f "$dist/$owner_marker" ]]; then
  echo "refusing to replace unowned dist path (missing $owner_marker): $dist" >&2
  exit 1
fi

stage="$(mktemp -d "$dist_parent/.microhs-web-dist.XXXXXX")"
cleanup() {
  if [[ -n "${stage:-}" && -d "$stage" ]]; then
    rm -rf -- "$stage"
  fi
}
trap cleanup EXIT

runtime_wasm="$repo/target/wasm32-unknown-unknown/release/microhs_runtime.wasm"
comb="$repo/generated/mhs.comb"
basepkg="$repo/generated/base.pkg"

echo "building embedded browser wasm (Rust cdylib; no emcc)"
# The dist ships only the Rust runtime wasm. emcc is used solely for the
# C-vs-Rust comparison bench (build-browser-bench.sh), never for the dist, so
# build the Rust wasm directly here rather than going through that script.
# --allow-undefined lets wasm-ld emit the mhs_host_*/mhs_js_* host bridge as
# imports (resolved by host.mjs at instantiation) instead of erroring.
RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=--allow-undefined" cargo build \
  --release \
  --manifest-path "$repo/rust/microhs-runtime/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --features embedded \
  --lib \
  --quiet

[[ -s "$basepkg" ]] || {
  echo "generated/base.pkg missing (run: make generated/base.pkg)" >&2
  exit 1
}

mkdir -p "$stage/include/lib"

cp "$runtime_wasm" "$stage/microhs_runtime.wasm"
cp "$here/compiler.mjs" "$stage/compiler.mjs"
cp "$here/host.mjs" "$stage/host.mjs"
cp "$comb" "$stage/mhs.comb"
cp "$basepkg" "$stage/base.pkg"
cp "$repo/LICENSE" "$stage/LICENSE"
cp -a "$repo/lib/." "$stage/include/lib/"
printf '%s\n' "MicroHs Rust browser distribution" > "$stage/$owner_marker"

echo "shipped base.pkg: $(wc -c < "$stage/base.pkg" | tr -d ' ') bytes"

DIST="$stage" node --input-type=module <<'NODE'
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
  // Preload each package at its vfs path and pass it to createCompiler({ packages })
  // (which adds -p<vfs>); library modules then load pre-typechecked from base.pkg.
  packages: [
    { dist: "base.pkg", vfs: "/base.pkg" },
  ],
};

await writeFile(path.join(dist, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
NODE

echo "running dist-only smoke"
DIST="$stage" node --input-type=module <<'NODE'
import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const dist = process.env.DIST;
const manifest = JSON.parse(await readFile(path.join(dist, "manifest.json"), "utf8"));
const { createCompiler } = await import(pathToFileURL(path.join(dist, "compiler.mjs")).href);
const files = {};

for (const [distRel, vfsPath] of Object.entries(manifest.includeFiles)) {
  files[vfsPath] = await readFile(path.join(dist, distRel));
}
// Preload the packages into the VFS at their declared paths.
for (const pkg of manifest.packages) {
  files[pkg.vfs] = await readFile(path.join(dist, pkg.dist));
}

const compiler = await createCompiler({
  wasm: await readFile(path.join(dist, "microhs_runtime.wasm")),
  comb: await readFile(path.join(dist, "mhs.comb")),
  files,
  packages: manifest.packages.map((p) => p.vfs),
});

try {
  const source = `module DistSmoke where
import Prelude
import Data.List

out :: Int
out = length (take 3 [1, 2, 3, 4, 5])
`;

  const { status, root, defs, error } = compiler.toCombinators(source, "out", { module: "DistSmoke" });
  if (status !== "ok" || root !== "DistSmoke.out" || !Array.isArray(defs) || defs.length === 0) {
    throw new Error(`dist smoke failed: status=${status} root=${root} defs=${defs?.length} error=${error}`);
  }
  console.log(`smoke: status=${status} root=${root} defs=${defs.length} (via base.pkg)`);
} finally {
  compiler.close();
}
NODE

rm -rf -- "$dist"
mv -- "$stage" "$dist"
stage=""

echo "dist manifest:"
while IFS= read -r file; do
  relative="${file#"$dist"/}"
  bytes="$(wc -c < "$file" | tr -d ' ')"
  printf '%s\t%s bytes\n' "$relative" "$bytes"
done < <(find "$dist" -type f | sort)
