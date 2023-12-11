import { transform } from '@swc/core';
import highlight from 'cli-highlight';

const DEMO_ESM =`
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

const DEMO_CJS = `
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

const transformWithPlugin = async (code: string) => {
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
  return result.code;
};

Promise.all([
  transformWithPlugin(DEMO_ESM),
  transformWithPlugin(DEMO_CJS),
]).then(([esmResult, cjsResult]) => {
  console.log('esm\n\n' + highlight(esmResult, { language: 'js' }));
  console.log('cjs\n\n' + highlight(cjsResult, { language: 'js' }));
});
