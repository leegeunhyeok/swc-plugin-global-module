import __re_export from "module";
export { default } from 'module';
global.__modules.init("test.js");
global.__modules.export("test.js", {
  default: __re_export
});
