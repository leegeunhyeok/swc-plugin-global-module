const _a = global.__modules.import("a");
const _b = global.__modules.import("b");
const __re_export_all = global.__modules.helpers.asWildcard(_a);
const __re_export_all1 = global.__modules.helpers.asWildcard(_b);
global.__modules.esm("test.js", {}, __re_export_all, __re_export_all1);
