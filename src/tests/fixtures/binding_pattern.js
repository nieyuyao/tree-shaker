export function main(unknown) {
  let { a } = { a: 1 };
  effect(a);

  let { b: c } = { b: 2 };
  effect(c);

  let { d = 3 } = { d: undefined };
  effect(d);

  let { [0 + 1] : e } = { 1: 4 };
  effect(e);

  // Destructing unknown has effect
  let { g: { h, i: { j } } } = unknown;
}