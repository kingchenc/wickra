// Remove the value re-exports that NAPI-RS emits for type-only declarations.
//
// When a `#[napi]` struct carries a `js_name`, the generator emits the class
// under that name and adds a TypeScript alias for the Rust name, so that
// `SmaNode` still resolves as a type:
//
//     export declare class SMA { ... }
//     export type SmaNode = SMA
//
// It then pushes BOTH names onto the list of runtime re-exports:
//
//     exports.push(def.name)
//     if (def.original_name && def.original_name !== def.name)
//       exports.push(def.original_name)
//
// The native module only registers the `js_name`, so the second line produces
// `module.exports.SmaNode = nativeBinding.SmaNode`, which is `undefined`. Every
// indicator here sets a `js_name`, so half of what the package exported was
// undefined -- 518 of 1038 names -- and `Object.keys(require("wickra"))` was
// mostly noise.
//
// This runs after `napi build` and drops those lines. It is driven by the
// generated `index.d.ts` rather than by a name pattern: a simple
// `export type A = B` line is exactly the alias form above, so anything the
// generator declares as a type stops being re-exported as a value. Alias names
// stay importable as types, which is what they always were.
import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), '..')
const dtsPath = join(packageRoot, 'index.d.ts')
const jsPath = join(packageRoot, 'index.js')

const dts = readFileSync(dtsPath, 'utf8')
const js = readFileSync(jsPath, 'utf8')

const typeOnly = new Set(
  [...dts.matchAll(/^export type ([A-Za-z0-9_$]+) = [A-Za-z0-9_$]+$/gm)].map((m) => m[1]),
)

const newline = js.includes('\r\n') ? '\r\n' : '\n'
const kept = []
const dropped = []
for (const line of js.split(/\r?\n/)) {
  const match = /^module\.exports\.([A-Za-z0-9_$]+) = nativeBinding\.[A-Za-z0-9_$]+$/.exec(line)
  if (match && typeOnly.has(match[1])) {
    dropped.push(match[1])
  } else {
    kept.push(line)
  }
}

writeFileSync(jsPath, kept.join(newline), 'utf8')
console.log(
  `pruned ${dropped.length} type-only re-export(s) from index.js` +
    ` (${typeOnly.size} type alias(es) declared in index.d.ts)`,
)
