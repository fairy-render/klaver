import { ReadableStream } from "@klaver/wintercg";

const count = 0;
const stream = new ReadableStream(
  {
    async pull(ctrl) {
      await new Promise((res) => Core.timers.createTimer(res, 100, false));
      ctrl.enqueue(`Hello, World ${count}`);

      // if (++count > 10) ctrl.close();
    },

    cancel(reason) {
      print("Reason", reason);
    },
  },
  {
    highWaterMark: 3,
  }
);

let n = 0;
for await (const chunk of stream) {
  print(chunk);
  if (n++ === 4) {
    print("Just because");
    break;
  }
}

const s = ReadableStream.from([1, 2, 3, 4]);

for await (const n of s) {
  print(n);
}

const v = ReadableStream.from(
  (function* () {
    for (const i of [5, 6, 7]) {
      yield i;
    }
  })()
);

for await (const n of v) {
  print(n);
}

const map = new Map();
map.set("Hello", "World");

const v2 = ReadableStream.from(map);

for await (const n of v2) {
  print(n);
}
