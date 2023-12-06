function fn() {}
global.__modules.init("test.js");
global.__modules.export("test.js", {
  default: fn
});
