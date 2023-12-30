import path from 'node:path';
import { transform } from '@swc/core';
import { faker } from '@faker-js/faker';
import '../index';
import type { CommonJsContext } from '../types';

const SNAPSHOT_TEST_CODE_INPUT_ESM = `
import React, { useState, useEffect } from 'react';
import { Container } from '@app/components';
import { useCustomHook } from '@app/hooks';
import { SECRET_KEY } from '@app/secret';
import * as app from '@app/core';

// named export & declaration
export function MyComponent(): JSX.Element {
  const [count, setCount] = useState(0);
  useCustomHook(SECRET_KEY);
  return <Container>{count}</Container>;
}

// named export with alias
export { app as AppCore };

// default export & anonymous declaration
export default class {}

// re-exports
export * from '@app/module_a';
export * from '@app/module_b';
export * as car from '@app/module_c';
export { driver as driverModule } from '@app/module_d';
`;

const SNAPSHOT_TEST_CODE_INPUT_CJS = `
const core = require('@app/core');
const utils = global.requireWrapper(require('@app/utils'));

if (process.env.NODE_ENV === 'production') {
  module.exports = class ProductionClass extends core.Core {};
} else {
  module.exports = class DevelopmentClass extends core.Core {};
}

exports.getReact = () => {
  return require('react');
};
`;

const transformWithPlugin = async (code: string, moduleId?: string) => {
  const result = await transform(code, {
    isModule: true,
    filename: 'demo.tsx',
    jsc: {
      target: 'esnext',
      parser: {
        syntax: 'typescript',
        tsx: true,
      },
      experimental: {
        plugins: [
          ['.', {
            moduleId,
            runtimeModule: true,
            externalPattern: '^(@app/secret)',
            moduleIds: {
              react: '456',
            },
          }],
        ],
      },
      externalHelpers: false,
    },
  });
  return result.code;
};

const generateModuleId = () => {
  return path.join(faker.system.directoryPath(), faker.word.noun() + '.js');
};

describe('swc-plugin-global-module/runtime', () => {
  const snapshot = process.env.CI === 'true' ? it.skip : it;

  snapshot('match snapshot (esm)', async () => {
    expect(
      await transformWithPlugin(SNAPSHOT_TEST_CODE_INPUT_ESM),
    ).toMatchSnapshot();
  });

  snapshot('match snapshot (cjs)', async () => {
    expect(
      await transformWithPlugin(SNAPSHOT_TEST_CODE_INPUT_CJS),
    ).toMatchSnapshot();
  });

  snapshot('match snapshot (esm + moduleId)', async () => {
    expect(
      await transformWithPlugin(SNAPSHOT_TEST_CODE_INPUT_ESM, 'esm-id'),
    ).toMatchSnapshot();
  });

  snapshot('match snapshot (cjs + moduleId)', async () => {
    expect(
      await transformWithPlugin(SNAPSHOT_TEST_CODE_INPUT_CJS, 'cjs-id'),
    ).toMatchSnapshot();
  });

  it('global object must expose apis', () => {
    expect(typeof global.__modules === 'object').toEqual(true);
    expect(typeof global.__modules.esm === 'function').toEqual(true);
    expect(typeof global.__modules.cjs === 'function').toEqual(true);
    expect(typeof global.__modules.import === 'function').toEqual(true);
    expect(typeof global.__modules.require === 'function').toEqual(true);
    expect(typeof global.__modules.helpers === 'object').toEqual(true);
  });

  describe('when trying to get unregistered module', () => {
    it('should throw error', () => {
      expect(() => {
        global.__modules.__registry['unregistered'];
      }).toThrowErrorMatchingSnapshot();
    });
  });

  describe('ES Modules', () => {
    let moduleId: string;
    let exportValue: string;
    let exports: Record<string, unknown>;

    beforeEach(() => {
      moduleId = generateModuleId();
      exportValue = faker.string.uuid();
    });

    describe('when register module that named export only', () => {
      let namedExportKey: string;

      beforeEach(() => {
        namedExportKey = faker.string.alpha(10);
        exports = { [namedExportKey]: exportValue };
        global.__modules.esm(moduleId, exports);
      });

      describe('when call `import()` to get registered module', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.import(moduleId);
          expect(targetModule[namedExportKey]).toEqual(exportValue);
        });
      });
    });

    describe('when register module that has default export', () => {
      beforeEach(() => {
        exports = { default: exportValue };
        global.__modules.esm(moduleId, exports);
      });

      describe('when call `import()` to get registered module', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.import(moduleId);
          expect(targetModule.default).toEqual(exportValue);
        });
      });

      describe('when wrap module with `asWildcard` helper', () => {
        it('should exclude `default` property', () => {
          const targetModule = global.__modules.import(moduleId);
          const wrappedModule = global.__modules.helpers.asWildcard(targetModule);
          expect(wrappedModule.default).toBeUndefined();
        });
      });
    });

    describe('re-exports', () => {
      let reExportModule: Record<string, unknown>;
      let namedExportKey: string;

      beforeEach(() => {
        namedExportKey = faker.string.alpha(10);
        reExportModule = { [namedExportKey]: exportValue };
        global.__modules.esm(moduleId, {}, reExportModule);
      });

      describe('when call `import()` to get registered module', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.import(moduleId);
          expect(targetModule[namedExportKey]).toEqual(exportValue);
        });
      });

      describe('when module that contains `default` property to be re-exported', () => {
        beforeEach(() => {
          reExportModule = { default: exportValue };
          global.__modules.esm(moduleId, {}, reExportModule);
        });

        describe('when call `import()` to get registered module', () => {
          it('should returns exported module with the `default` property excluded', () => {
            const targetModule = global.__modules.import(moduleId);
            expect(targetModule.default).toBeUndefined();
          });
        });
      });
    });
  });

  describe('CommonJS', () => {
    let moduleId: string;
    let exportValue: string;

    beforeEach(() => {
      moduleId = generateModuleId();
      exportValue = faker.string.uuid();
    });

    describe('when register module via cjs context', () => {
      let context: CommonJsContext;

      describe('named exports', () => {
        let namedExportKey: string;

        beforeEach(() => {
          namedExportKey = faker.string.alpha(10);
          context = global.__modules.cjs(moduleId);
          context.exports[namedExportKey] = exportValue;
        });
  
        describe('when call `require()` to get registered module', () => {
          it('should returns exported module', () => {
            const targetModule = global.__modules.require(moduleId);
            expect(targetModule[namedExportKey]).toEqual(exportValue);
          });
        });
      });

      describe('default exports', () => {
        beforeEach(() => {
          context = global.__modules.cjs(moduleId);
          context.exports.default = exportValue;
        });
  
        describe('when call `require()` to get registered module', () => {
          it('should returns exported module without `default` key', () => {
            const targetModule = global.__modules.require(moduleId);
            expect(targetModule).toEqual(exportValue);
            expect(targetModule.default).toBeUndefined();
          });
        });
      });
    });
  });

  describe('interoperability of ES Modules and CommonJS', () => {
    let esModuleId: string;
    let commonJsModuleId: string;

    beforeEach(() => {
      esModuleId = generateModuleId();
      commonJsModuleId = generateModuleId();
    });
    
    describe('default exports', () => {
      let defaultExportValue: string;

      beforeEach(() => {
        defaultExportValue = faker.string.uuid();

        // ESM
        global.__modules.esm(esModuleId, {
          default: defaultExportValue,
        });
  
        // CJS
        const context = global.__modules.cjs(commonJsModuleId);
        context.exports.default = defaultExportValue;
      });
  
      describe('when call `import()` to get CommonJS module', () => {
        it('should returns exported module with `default` key', () => {
          const targetModule = global.__modules.import(commonJsModuleId);
          expect(targetModule.default).toEqual(defaultExportValue);
        });
      });
  
      describe('when call `require()` to get ES modules', () => {
        it('should returns exported module with `default` key', () => {
          const targetModule = global.__modules.require(esModuleId);
          expect(targetModule.default).toEqual(defaultExportValue);
        });
      });
    });
    
    describe('named exports', () => {
      let namedExportKey: string;
      let namedExportValue: string;

      beforeEach(() => {
        namedExportKey = faker.string.alpha(10);
        namedExportValue = faker.string.uuid();

        // ESM
        global.__modules.esm(esModuleId, {
          [namedExportKey]: namedExportValue,
        });
  
        // CJS
        const context = global.__modules.cjs(commonJsModuleId);
        context.exports[namedExportKey] = namedExportValue;
      });
  
      describe('when call `import()` to get CommonJS module', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.import(commonJsModuleId);
          expect(targetModule[namedExportKey]).toEqual(namedExportValue);
        });
      });
  
      describe('when call `require()` to get ES modules', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.require(esModuleId);
          expect(targetModule[namedExportKey]).toEqual(namedExportValue);
        });
      });
    });
  });

  describe('external modules', () => {
    const MODULE_ID = 'external';

    describe('when register module to external registry', () => {
      let exportKey: string;
      let exportValue: string;

      beforeEach(() => {
        exportKey = faker.string.alpha(10);
        exportValue = faker.string.uuid();
        global.__modules.external(MODULE_ID, {
          [exportKey]: exportValue,
        });
      });

      describe('when call `external()` to get external module', () => {
        it('should returns exported module', () => {
          const targetModule = global.__modules.external(MODULE_ID);
          expect(targetModule[exportKey]).toEqual(exportValue);
        });
      });

      describe('when call `import()` to get external module', () => {
        it('should throw error', () => {
          expect(() => {
            global.__modules.import(MODULE_ID);
          }).toThrowErrorMatchingSnapshot();
        });
      });

      describe('when call `require()` to get external module', () => {
        it('should throw error', () => {
          expect(() => {
            global.__modules.require(MODULE_ID);
          }).toThrowErrorMatchingSnapshot();
        });
      });
    });
  });
});
