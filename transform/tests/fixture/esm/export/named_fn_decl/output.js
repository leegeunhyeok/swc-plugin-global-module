function namedFunction() {
  console.log('body');
}
global.__modules.esm("test.js", { namedFunction });
