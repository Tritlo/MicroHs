/** Scalar types in compiler-generated JavaScript import metadata. */
export type JavascriptScalar = "Int" | "Word" | "Int64" | "Word64" | "Float" | "Double";

/** One synchronous JavaScript body. Parameters retain the existing $0 notation. */
export interface JavascriptBinding {
  module: "javascript";
  name: string;
  source: string;
  parameters: JavascriptScalar[];
  result: JavascriptScalar | "Unit";
}

/** Contents of the .imports.json companion produced by -oPROGRAM.wat. */
export interface JavascriptImportManifest {
  version: 1;
  bindings: JavascriptBinding[];
}

/**
 * Compile generated JavaScript bodies into typed WebAssembly host imports.
 * Int64 and Word64 use bigint. Other scalar values use number. Word arguments
 * retain their unsigned interpretation. Bodies execute synchronously and can
 * access the host global object. Load metadata only from trusted programs.
 */
export function createJavascriptImports(manifest: JavascriptImportManifest): WebAssembly.Imports {
  if (manifest.version !== 1 || !Array.isArray(manifest.bindings)) {
    throw new Error("Unsupported JavaScript import metadata");
  }
  const imports: WebAssembly.ModuleImports = Object.create(null);
  const scalarTypes = new Set(["Int", "Word", "Int64", "Word64", "Float", "Double"]);
  for (const binding of manifest.bindings) {
    if (binding.module !== "javascript" || typeof binding.name !== "string" ||
        typeof binding.source !== "string" || !Array.isArray(binding.parameters) ||
        binding.parameters.length > 6 || binding.parameters.some((type) => !scalarTypes.has(type)) ||
        (binding.result !== "Unit" && !scalarTypes.has(binding.result))) {
      throw new Error("Invalid JavaScript import binding");
    }
    if (Object.hasOwn(imports, binding.name)) {
      throw new Error(`Duplicate JavaScript import: ${binding.name}`);
    }
    const body = new Function(...binding.parameters.map((_, index) => `$${index}`),
      `"use strict";\n${binding.source}`) as (...args: (number | bigint)[]) => unknown;
    imports[binding.name] = (...args: (number | bigint)[]): number | bigint | undefined => {
      const values = args.map((value, index) => {
        switch (binding.parameters[index]) {
          case "Word": return Number(value) >>> 0;
          case "Word64": return BigInt.asUintN(64, BigInt(value));
          default: return value;
        }
      });
      const result = body(...values);
      if (result instanceof Promise) {
        throw new TypeError(`JavaScript import must be synchronous: ${binding.name}`);
      }
      if (binding.result === "Unit") return undefined;
      const expected = binding.result === "Int64" || binding.result === "Word64" ? "bigint" : "number";
      if (typeof result !== expected) {
        throw new TypeError(`JavaScript import ${binding.name} must return ${expected}`);
      }
      return result as number | bigint;
    };
  }
  return { javascript: imports };
}
