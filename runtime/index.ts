import { createModule, createModuleRegistry } from './helpers';
import type { GlobalModule, GlobalModuleApi } from './types';

((global) => {
  if (typeof global === 'undefined') {
    throw new Error('[Global Module] `global` is undefined');
  }

  const registry = createModuleRegistry();
  const globalModuleApi: GlobalModuleApi = {
    registry,
    esm: (moduleId, exportedModule, ...reExportedModules) => {
      const module = createModule(exportedModule);
      let exported = 0;
  
      Object.getOwnPropertyNames(exportedModule).forEach((exportName) => {
        Object.defineProperty(module, exportName, {
          enumerable: true,
          get: () => exportedModule[exportName],
        });
        ++exported;
      });
  
      reExportedModules.forEach((reExportedModule) => {
        Object.getOwnPropertyNames(reExportedModule).forEach((exportName) => {
          if (exportName !== 'default') {
            Object.defineProperty(module, exportName, {
              enumerable: true,
              get: () => reExportedModule[exportName],
            });
            ++exported;
          }
        });
      });

      if (exported === 0) {
        throw new Error(`[Global Module] no exports found in '${moduleId}'`)
      }
  
      registry[moduleId] = module;
    },
    helpers: {
      asWildcard: (targetModule: GlobalModule) => {
        const newModule = createModule();
        Object.getOwnPropertyNames(targetModule).forEach((exportName) => {
          if (exportName !== 'default') {
            const descriptor = Object.getOwnPropertyDescriptor(targetModule, exportName);
            Object.defineProperty(
              newModule,
              exportName,
              descriptor ?? {
                enumerable: true,
                get: () => targetModule[exportName],
              },
            );
          }
        });
        return newModule;
      }
      
    }
  };

  Object.defineProperty(global, '__modules', { value: globalModuleApi });

  // Define `global` property to global object.
  if (!('global' in global)) {
    Object.defineProperty(global, 'global', { value: global });
  }
})(
  typeof globalThis !== 'undefined'
    ? globalThis
    : typeof global !== 'undefined'
    ? global
    : typeof window !== 'undefined'
    ? window
    : this,
);
