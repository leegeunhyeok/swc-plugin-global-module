# swc-plugin-global-module

> [!WARNING]
> This plugin is for custom module system to implement Hot Module Replacement(HMR) in some bundlers that don't support it.

## Installation

```bash
npm install swc-plugin-global-module
# or yarn
yarn add swc-plugin-global-module
```

## Usage

Inject runtime script to top of bundle for using [Global module APIs](./runtime/types.ts).

```ts
import 'swc-plugin-global-module/runtime';

// Now you can use global module API (global.__modules)
```

and add plugin to your swc options.

```ts
import { transform } from '@swc/core';

await transform(code, {
  jsc: {
    experimental: {
      plugins: [
        // Add plugin here.
        ['swc-plugin-global-module', {
          /**
           * Convert import statements to custom module system and remove export statements.
           *
           * Defaults to `false`.
           */
          runtimeModule: true,
          /**
           * Actual module path aliases (resolved module path)
           *
           * Defaults to none.
           */
          importPaths: {
            "<import source>": "actual module path",
            // eg. react
            "react": "node_modules/react/cjs/react.development.js",
          },
        }],
      ],
    },
    /**
     * You should disable external helpers when `runtimeModule` is `true`
     * because external helper import statements will be added after plugin transformation.
     */
    externalHelpers: false,
  },
});
```

## Preview

Before

```tsx
import React, { useState, useEffect } from 'react';
import { Container, Section, Button, Text } from '@app/components';
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
```

After

```js
// with `runtimeModule: true`
const __app_components = global.__modules.registry["@app/components"];
const __app_core = global.__modules.registry["@app/core"];
const __app_hooks = global.__modules.registry["@app/hooks"];
const __app_module_a = global.__modules.registry["@app/module_a"];
const __app_module_b = global.__modules.registry["@app/module_b"];
const __app_module_c = global.__modules.registry["@app/module_c"];
const __app_module_d = global.__modules.registry["@app/module_d"];
const _react = global.__modules.registry["react"];
const React = _react.default;
const useState = _react.useState;
const Container = __app_components.Container;
const useCustomHook = __app_hooks.useCustomHook;
const app = global.__modules.helpers.asWildcard(__app_core);
const __re_export_all = global.__modules.helpers.asWildcard(__app_module_a);
const __re_export_all1 = global.__modules.helpers.asWildcard(__app_module_b);
const __re_export = global.__modules.helpers.asWildcard(__app_module_c);
const __re_export1 = __app_module_d.driver;
function MyComponent() {
    const [count, setCount] = useState(0);
    useCustomHook(app);
    return /*#__PURE__*/ React.createElement(Container, null, count);
}
class __Class {
}
global.__modules.esm("demo.tsx", {
  MyComponent,
  AppCore: app,
  default: __Class,
  car: __re_export,
  driverModule: __re_export1
}, __re_export_all, __re_export_all1);
```

## Use Cases

<details>

  <summary>esbuild</summary>

  ```ts
  import fs from 'node:fs/promises';
  import path from 'node:path';
  import * as esbuild from 'esbuild';
  import { transform } from '@swc/core';

  const ROOT = path.resolve('.');

  const context = await esbuild.context({
    // ...,
    sourceRoot: ROOT,
    metafile: true,
    inject: ['swc-plugin-global-module/runtime'],
    plugins: [
      // ...,
      {
        name: 'store-metadata-plugin',
        setup(build) {
          build.onEnd((result) => {
            /**
             * Store metafile data to memory for read it later.
             * 
             * # Metafile
             *
             * ```js
             * {
             *   inputs: {
             *     'src/index.ts': {
             *       bytes: 100,
             *       imports: [
             *         {
             *           kind: '...',
             *           // Import path in source code
             *           original: 'react',
             *           // Resolved path by esbuild (actual module path)
             *           path: 'node_modules/react/cjs/react.development.js',
             *           external: false,
             *         },
             *         ...
             *       ],
             *     },
             *     ...
             *   },
             *   outputs: {...}
             * }
             * ```
             */
            store.set('metafile', result.metafile);
          });
        },
      },
    ],
  });
  await context.rebuild();

  // eg. file system watcher
  watcher.addEventListener(async ({ path }) => {
    /**
     * Get import paths from esbuild's metafile data.
     *
     * # Return value
     *
     * ```js
     * {
     *   'react': 'node_modules/react/cjs/react.development.js',
     *   'src/components/Button': 'src/components/Button.tsx',
     *   ...
     * }
     * ```
     */
    const getImportPathsFromMetafile = (filepath: string) => {
      const metafile = store.get('metafile');
      return metafile?.inputs[filepath]?.imports?.reduce((prev, curr) => ({
        ...prev,
        [curr.original]: curr.path
      }), {}) ?? {};
    };

    const strippedPath = path.replace(ROOT, '').substring(1);
    const rawCode = await fs.readFile(path, 'utf-8');
    const transformedCode = await transform(rawCode, {
      filename: strippedPath,
      jsc: {
        experimental: {
          plugins: [
            ['swc-plugin-global-module', {
              runtimeModule: true,
              importPaths: getImportPathsFromMetafile(strippedPath),
            }],
          ],
        },
        externalHelpers: false,
      },
    });
    
    // eg. send HMR update message to clients via websocket.
    sendHMRUpdateMessage(path, transformedCode);
  });
  ```

</details>

## License

[MIT](./LICENSE)
