export function f1() {
  let a = 1;
  a = 2;
  return a;
}

export function f2(a) {
  a = 2;
  return a;
}

export function f3(a, b) {
  if (b) a = 2;
  return a;
}

export function f4(a, b) {
  if (b) a = 2;
  a = 3
  return a;
}

export function f5(a, b) {
  a = b;
  return a;
}

export function f6(a, b) {
  a.p = b;
  global.p = 1;
  (1).p = 2;
  (1).p = effect;
  return a;
}
