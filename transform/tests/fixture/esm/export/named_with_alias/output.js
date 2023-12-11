const _module = global.__modules.import("module");
const __re_export = global.__modules.helpers.asWildcard(_module);
global.__modules.esm("test.js", {
  rename: __re_export
});
