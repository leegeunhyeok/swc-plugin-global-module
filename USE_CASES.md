# Use Cases

## esbuild

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
            moduleIds: getImportPathsFromMetafile(strippedPath),
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
