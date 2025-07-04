const formatter = new Intl.DateTimeFormat("en-gb", {
  //   timeZone: "Atlantic/Reykjavik",
  //   dateStyle: "full",
  //   timeStyle: "full",
  //   year: "2-digit",
  weekday: "short",
  day: "numeric",
  //   month: "narrow",
  hour: "2-digit",
  minute: "2-digit",
  year: "numeric",
  month: "long",
  timeZoneName: "longGeneric",
});

console.log(formatter.format(new Date()));

// console.log(formatter.calendar());

const number = new Intl.NumberFormat("da");

console.log(number.format(20202));

function inject() { }

try {
  @inject
  class Test { }
} catch (e) {
  console.log(e);
}

export async function test() {
  using file = new File();
  console.log("Asyn function");
  return await inject();
}

function* test2() { }

Symbol.dispose = Symbol("dispose")


console.log(Symbol.dispose)