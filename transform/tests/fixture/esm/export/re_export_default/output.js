const _module = global.__modules.registry["module"];
const __re_export = _module.default;
global.__modules.esm("test.js", {
  default: __re_export
});
