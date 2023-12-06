import __re_export from "a";
import __re_export1 from "b";
export { default as A } from 'a';
export { default as B } from 'b';
global.__modules.init("test.js");
global.__modules.export("test.js", {
  A: __re_export,
  B: __re_export1
});
