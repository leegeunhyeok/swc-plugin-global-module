import { obj, createModuleRegistry } from './helpers';
import type { GlobalModule, GlobalModuleApi } from './types';

((global) => {
  if (typeof global === 'undefined') {
    throw new Error('[Global Module] `global` is undefined');
  }

  const __defProp = Object.defineProperty;
  const __getOwnPropNames = Object.getOwnPropertyNames;
  const __getOwnPropDesc = Object.getOwnPropertyDescriptor;
  const __hasOwnProp = Object.prototype.hasOwnProperty;

  const __copyProps = (
    to: any,
    from: any,
    except?: string,
    desc?: PropertyDescriptor,
  ) => {
    if ((from && typeof from === 'object') || typeof from === 'function') {
      for (const key of __getOwnPropNames(from)) {
        if (!__hasOwnProp.call(to, key) && key !== except) {
          __defProp(to, key, {
            get: () => from[key],
            enumerable:
              !(desc = __getOwnPropDesc(from, key)) || desc.enumerable,
          });
        }
      }
    }
    return to;
  };

  const registry = createModuleRegistry();
  const globalModuleApi: GlobalModuleApi = {
    __registry: registry,
    esm: (moduleId, exportedModule, ...reExportedModules) => {
      const esModule = __copyProps(obj(exportedModule), exportedModule);
      reExportedModules.forEach((reExportedModule) => {
        __copyProps(esModule, reExportedModule, 'default');
      });
      registry[moduleId] = esModule;
    },
    cjs: (moduleId) => {
      const commonJsModule = (registry[moduleId] = __defProp(obj(), '__cjs', {
        enumerable: true,
        value: true,
      }));
      return { exports: commonJsModule };
    },
    import: (moduleId) => registry[moduleId],
    require: (moduleId) => {
      const targetModule = registry[moduleId];
      return targetModule.__cjs
        ? targetModule.default ?? targetModule
        : targetModule;
    },
    helpers: {
      asWildcard: (targetModule: GlobalModule) => {
        return __copyProps(obj(), targetModule, 'default');
      },
    },
  };

  __defProp(global, '__modules', { value: globalModuleApi });

  // Define `global` property to global object.
  if (!('global' in global)) {
    __defProp(global, 'global', { value: global });
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
