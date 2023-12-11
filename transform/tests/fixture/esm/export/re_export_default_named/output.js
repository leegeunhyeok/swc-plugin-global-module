const _a = global.__modules.import("a");
const _b = global.__modules.import("b");
const __re_export = _a.default;
const __re_export1 = _b.default;
global.__modules.esm("test.js", {
  A: __re_export,
  B: __re_export1
});
