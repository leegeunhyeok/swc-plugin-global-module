export default function __fn() {}
global.__modules.init("test.js");
global.__modules.export("test.js", {
  default: __fn
});
