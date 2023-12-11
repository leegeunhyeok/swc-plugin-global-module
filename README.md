# swc-plugin-global-module

> [!WARNING]
> This plugin is for custom module system to implement Hot Module Replacement(HMR) in some bundlers that don't support it.

## Features

- 🌍 Register ESM and CJS to global module registry.
- 🏃 Runtime mode
  - Keep original `import` and `require` statements.
  - Transform to global module registry's `import` and `require` statements.

## How it works?

For more details, go to [MECHANISMS.md](MECHANISMS.md).

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
// ESM
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

```js
// CJS
const core = require('@app/core');
const utils = global.requireWrapper(require('@app/utils'));

if (process.env.NODE_ENV === 'production') {
  // Default exports (dynamic)
  module.exports = class ProductionClass extends core.Core {};
} else {
  // Default exports (dynamic)
  module.exports = class DevelopmentClass extends core.Core {};
}

// Named exports
exports.getReact = () => {
  // Dynamic require
  return require('react');
};
```

After

```js
const __app_components = global.__modules.import("@app/components");
const __app_core = global.__modules.import("@app/core");
const __app_hooks = global.__modules.import("@app/hooks");
const __app_module_a = global.__modules.import("@app/module_a");
const __app_module_b = global.__modules.import("@app/module_b");
const __app_module_c = global.__modules.import("@app/module_c");
const __app_module_d = global.__modules.import("@app/module_d");
const _react = global.__modules.import("node_modules/react/cjs/react.development.js");
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

```js
const __cjs = global.__modules.cjs("demo.tsx");
const core = global.__modules.require("@app/core");
const utils = global.requireWrapper(global.__modules.require("@app/utils"));
if (process.env.NODE_ENV === 'production') {
  module.exports = __cjs.exports.default = class ProductionClass extends core.Core {
  };
} else {
  module.exports = __cjs.exports.default = class DevelopmentClass extends core.Core {
  };
}
exports.getReact = __cjs.exports.getReact = ()=>{
  return global.__modules.require("react");
};
```

## Use Cases

Go to [USE_CASES.md](./USE_CASES.md).

## License

[MIT](./LICENSE)
