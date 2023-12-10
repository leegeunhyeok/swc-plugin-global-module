import path from 'node:path';
import { transform } from '@swc/core';
import { faker } from '@faker-js/faker';
import '../index';

const SNAPSHOT_TEST_CODE_INPUT = `
import React, { useState, useEffect } from 'react';
import { Container } from '@app/components';
import { useCustomHook } from '@app/hooks';
import * as app from '@app/core';

// named export & declaration
export function MyComponent(): JSX.Element {
  const [count, setCount] = useState(0);
  useCustomHook(app);
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

const generateModuleId = () => {
  return path.join(faker.system.directoryPath(), faker.word.noun() + '.js');
};

describe('swc-plugin-global-module/runtime', () => {
  const snapshot = process.env.CI === 'true' ? it.skip : it;

  it('global object must expose apis', () => {
    expect(typeof global.__modules === 'object').toEqual(true);
    expect(typeof global.__modules.esm === 'function').toEqual(true);
    expect(typeof global.__modules.helpers === 'object').toEqual(true);
  });

  snapshot('match snapshot', async () => {
    const { code: outputCode } = await transform(SNAPSHOT_TEST_CODE_INPUT, {
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
              runtimeModule: true,
              importPaths: {
                react: 'node_modules/react/cjs/react.development.js',
              },
            }],
          ],
        },
        externalHelpers: false,
      },
    });
    expect(outputCode).toMatchSnapshot();
  });

  describe('esm', () => {
    let moduleId: string;

    beforeEach(() => {
      moduleId = generateModuleId();
    });

    describe('when trying to get unregistered module', () => {
      it('should throw error', () => {
        expect(() => global.__modules.import(faker.string.alpha())).toThrow(Error);
      });
    });

    describe('register modules', () => {
      let exportValue: string;
      let exports: Record<string, unknown>;

      describe('module that named export only', () => {
        let namedExportKey: string;

        beforeEach(() => {
          namedExportKey = faker.string.alpha(10);
          exportValue = faker.string.uuid();
          exports = { [namedExportKey]: exportValue };
          global.__modules.esm(moduleId, exports);
        });
  
        describe('when trying to get registered module', () => {
          it('should returns exported module', () => {
            const exportedModule = global.__modules.registry[moduleId];
            expect(exportedModule[namedExportKey]).toEqual(exportValue);
          });
        });
      });

      describe('module that has default export', () => {
        const EXPORT_KEY = 'default';

        beforeEach(() => {
          exportValue = faker.string.uuid();
          exports = { [EXPORT_KEY]: exportValue };
          global.__modules.esm(moduleId, exports);
        });
  
        describe('when trying to get registered module', () => {
          it('should returns exported module', () => {
            const exportedModule = global.__modules.registry[moduleId];
            expect(exportedModule[EXPORT_KEY]).toEqual(exportValue);
          });
        });

        describe('when wrap module with `asWildcard` helper', () => {
          it('should exclude `default` property', () => {
            const exportedModule = global.__modules.registry[moduleId];
            const wrappedModule = global.__modules.helpers.asWildcard(exportedModule);
            expect(wrappedModule.default).toBeUndefined();
          });
        });
      });

      describe('re-exports', () => {
        let reExportModule: Record<string, unknown>;
        let namedExportKey: string;
  
        beforeEach(() => {
          namedExportKey = faker.string.alpha(10);
          exportValue = faker.string.uuid();
          reExportModule = { [namedExportKey]: exportValue };
          global.__modules.esm(moduleId, {}, reExportModule);
        });
  
        describe('when trying to get registered module', () => {
          it('should returns exported module', () => {
            const exportedModule = global.__modules.registry[moduleId];
            expect(exportedModule[namedExportKey]).toEqual(exportValue);
          });
        });
  
        describe('when module that contains `default` property to be re-exported', () => {
          const EXPORT_KEY = 'default';
          let namedExportKey: string;

          beforeEach(() => {
            namedExportKey = faker.string.alpha(10);
            exportValue = faker.string.uuid();
            reExportModule = {
              [EXPORT_KEY]: exportValue,
              namedExportKey: null,
            };
            global.__modules.esm(moduleId, {}, reExportModule);
          });
  
          describe('when call `import()` with the exported module', () => {
            it('should exclude `default` property', () => {
              const exportedModule = global.__modules.registry[moduleId];
              expect(exportedModule[EXPORT_KEY]).toBeUndefined();
            });
          });
        });
      });
    });
  });
});
