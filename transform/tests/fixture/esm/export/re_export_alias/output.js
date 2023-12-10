const _module = global.__modules.registry["module"];
const __re_export = global.__modules.helpers.asWildcard(_module);
global.__modules.esm("test.js", {
  rename: __re_export
});
