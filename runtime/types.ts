export type GlobalModuleId = string;
export type GlobalModuleRegistry = Record<GlobalModuleId, GlobalModule>;
export type GlobalModule<T = any> = T;

export interface CommonJsContext {
  exports: GlobalModule;
}

export interface GlobalModuleApi {
  __registry: GlobalModuleRegistry;
  __externalRegistry: GlobalModuleRegistry;
  /**
   * Register an ESM module to global registry.
   * 
   * ```js
   * esm('module_id', exports, ...reExports);
   * // will be registered,
   * reg = {
   *   ...exports,
   *   ...reExports[n], // All of re-exports and `default` property will be excluded.
   * };
   * ```
   */
  esm: (
    id: GlobalModuleId,
    exportedModule: GlobalModule,
    ...reExportedModules: GlobalModule[]
  ) => void;
  /**
   * Returns a CommonJS module context to register a CommonJS module to global registry.
   * 
   * ```js
   * const ctx = cjs('module_id');
   *
   * // 1. Exports as default.
   * module.exports = ctx.exports = bar;
   * // will be registered,
   * reg = { default: baz }; // For ESM interoperability. 
   *
   * // 2. Named exports.
   * exports.named = ctx.exports.named = foo;
   * // will be registered,
   * reg = { named: foo };
   * ```
   */
  cjs: (id: GlobalModuleId) => CommonJsContext,
  /**
   * Register module as external to global module registry.
   * 
   * - Provide `source` only: get module from global registry.
   * - Provide `source` and `module`: register module to global registry.
   * 
   * ```ts
   * // register module
   * import * as __external from 'react';
   * external('react', __external);
   * 
   * // get module
   * const fromGlobal = external('react');
   * ```
   */
  external: (source: string, module?: GlobalModule) => GlobalModule,
  import: (id: GlobalModuleId) => GlobalModule,
  require: (id: GlobalModuleId) => GlobalModule,
  helpers: {
    /**
     * Helper for `import *` (exclude `default` property).
     */
    asWildcard: (targetModule: GlobalModule) => GlobalModule;
  },
};
