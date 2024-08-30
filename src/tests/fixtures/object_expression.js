export function main() {
  let obj1 = { a: 1, unused: 2, b: { c: 3, unused: 4 } };
  effect(obj1.a, obj1.b.c);

  let obj2 = { unused: effect() }
  obj2 = 1;

  let obj3 = {
    a: 1,
    get a() {
      return 2
    },
    get b() {
      effect()
    },
    set a(value) {
      effect(value);
    }
  }
  effect(obj3.a);
  obj3.b;
}
