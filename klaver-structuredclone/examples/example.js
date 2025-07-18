

// const out = structuredClone({
//   test: 'string',
//   date: new Map
// })

function isObject(o, strict = true) {
  if (o === null || o === undefined) {
    return false;
  }
  const instanceOfObject = o instanceof Object;
  const typeOfObject = typeof o === 'object';
  const constructorUndefined = o.constructor === undefined;
  const constructorObject = o.constructor === Object;
  const typeOfConstructorObject = typeof o.constructor === 'function';
  let r;
  if (strict === true) {
    r = (instanceOfObject || typeOfObject) && (constructorUndefined || constructorObject);
  } else {
    r = (constructorUndefined || typeOfConstructorObject);
  }
  return r;
};

const test = new TestClass("Rasmus");

print(isPlainObject(test))
print(isPlainObject(new Date))
print(isPlainObject({}))
const date = structuredClone({
  date: new Date,
  value: 20,
  test: 'eeww',
  class: test
});
print(date)
print(date.class === test)
print(test === test)