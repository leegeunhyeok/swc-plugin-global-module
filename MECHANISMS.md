# Mechanisms

```mermaid
flowchart TD
    start([Build with SWC])
    subgraph Non-runtime
        direction TB
        a1[Code] --> b1{Has ESM\nmodule statements?}
        b1 -->|Yes| c1[Collect Imports & Exports]
        c1 --> d1[Add global export statements to code]
        d1 --> e1{Has CJS\nmodule expressions?}
        e1 -->|Yes| f1[Transform to assign to global exports expressions]

        b1 -->|No| e1
        e1 -->|No| g1
        f1 --> g1[Transformed Code]
    end
    subgraph Runtime
        direction TB
        a2[Code] --> b2{Has ESM\nmodule statements?}
        b2 -->|Yes| c2[Collect Imports & Exports]
        c2 --> d2[Transform import statements\nto global import statements]
        d2 --> e2[Add global export statements to code]
        e2 --> f2{Has CJS\nmodule expressions?}
        f2 -->|Yes| g2[Transform to global require call expressions]
        g2 --> h2[Transform to assign to global exports expressions]

        b2 -->|No| f2
        f2 -->|No| i2
        h2 --> i2[Transformed Code]
    end
    start-->Non-runtime
    start-->Runtime
    Non-runtime-->fin[End]
    Runtime-->fin
```

## Non-runtime

```mermaid
flowchart TB
    subgraph Module A
    direction LR
    a1[exports]
    b1[exports]
    c1[exports]
    end
    subgraph Module B
    direction LR
    a2[exports]
    b2[exports]
    c2[exports]
    end
    reg[Global Registry]

    a1-->reg
    a2-->reg
    b1-->reg
    b2-->reg
    c1-->reg
    c2-->reg
```

This mode will be added global export statements only.

It is appropriate to enable this option when first bundling because modules contained inside the bundle can be registered to the global module registry.

Non-runtime mode can be enabled with `runtimeModule: false`.

```ts
// ESM
import App from '@app/core';
import { services } from '@app/services';
import * as helpers from '@app/helpers';

const app = new App();

export function initialize() {
  // ...
};

export default app;
```

```ts
// CJS
const App = require('@app/core');
const { services } = require('@app/services');
const helpers = require('@app/helpers');

const app = new App();

exports.initialize = function() {
  // ...
};

module.exports = app;
```

To

```diff
import App, { services } from '@app/core';
import * as helpers from '@app/helpers';

const app = new App();

export function initialize() {
  // ...
};

export default app;

+ global.__modules.esm('main.ts', {
+   default: app,
+   initialize,
+ });
```

```diff
+ const __cjs = global.__modules.cjs("main.ts");
const App = require('@app/core');
const { services } = require('@app/services');
const helpers = require('@app/helpers');

const app = new App();

- exports.initialize = function() {
+ exports.initialize = __cjs.exports.initialize = function() {
  // ...
};

- module.exports = app;
+ module.exports = __cjs.exports.default = app;
```

Original module statements will be handled by bundlers(like an esbuild).

## Runtime

```mermaid
flowchart TB
    subgraph Module A
    direction LR
    i1[imports]
    a1[exports]
    b1[exports]
    c1[exports]
    end
    subgraph Module B
    direction LR
    i2[imports]
    a2[exports]
    b2[exports]
    c2[exports]
    end
    reg[Global Registry]

    reg-->i1
    reg-->i2
    a1-->reg
    a2-->reg
    b1-->reg
    b2-->reg
    c1-->reg
    c2-->reg
```

This mode will be transform `import` and `require` statements into global module apis(and also including adding global export statements).

It is appropriate to enable this option when need to apply changes on runtime like a HMR(Hot Module Replacement).

Runtime mode can be enabled with `runtimeModule: true`.

```ts
// ESM
import App from '@app/core';
import { services } from '@app/services';
import * as helpers from '@app/helpers';

const app = new App();

export function initialize() {
  // ...
};

export default app;
```

```ts
// CJS
const App = require('@app/core');
const { services } = require('@app/services');
const helpers = require('@app/helpers');

const app = new App();

exports.initialize = function() {
  // ...
};

module.exports = app;
```

To

```diff
- import App from '@app/core';
- import { services } from '@app/services';
- import * as helpers from '@app/helpers';
+ const __app_core = global.__modules.import('@app/core');
+ const __app_services = global.__modules.import('@app/services');
+ const __app_helpers = global.__modules.import('@app/helpers');
+ const App = __app_core.default;
+ const services = __app_services.services;
+ const helpers = global.__modules.helpers.asWildcard(__app_helpers);

const app = new App();

function initialize() {
  // ...
}

- export default app;
+ const __export_default = app;

+ global.__modules.esm('demo.tsx', {
+   default: __export_default,
+   initialize,
+ });
```

```diff

- const App = require('@app/core');
- const { services } = require('@app/services');
- const helpers = require('@app/helpers');

const app = new App();

exports.initialize = function() {
  // ...
};

module.exports = app;

+ const __cjs = global.__modules.cjs('demo.tsx');
- const App = require('@app/core');
- const { services } = require('@app/services');
- const helpers = require('@app/helpers');
+ const App = global.__modules.require('@app/core');
+ const { services } = global.__modules.require('@app/services');
+ const helpers = global.__modules.require('@app/helpers');

const app = new App();

- exports.initialize = function() {
+ exports.initialize = __cjs.exports.initialize = function() {
  // ...
};

- module.exports = app;
+ module.exports = __cjs.exports.default = app;
```

When running sources on runtime, now `import` and `require` other modules from the global module registry.
