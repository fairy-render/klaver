// import { CountQueuingStrategy, WritableStream, ReadableStream, WritableStreamDefaultController, Console, EventTarget, AbortSignal, MessageChannel, structuredClone } from 'quick:base'

const output = []


const chan = new MessageChannel

chan.port1.start();

chan.port1.addEventListener('message', (e) => {
  console.log('message', e.data);
  chan.port1.close()
})

chan.port2.postMessage("HEllo")


const console = new Console((level, msg) => {
  print(`[${level}] ${msg}`)
})

console.log(typeof MessageChannel)

console.log(new AbortSignal() instanceof EventTarget);

const date = new Date();

const test = {
  hello: "world",
  date: date,
  sub: {
    date
  }
}

test.self = test;

const copy = structuredClone(test);

console.log(test);

// const test = new WritableStreamDefaultController({

// })

console.debug("rapra")

const writeStream = new WritableStream({
  start: () => {
    print("Started")
  },
  async write(chunk) {
    print('write')
    output.push(chunk)
    print("write done")
  },
  close() {
    print("Close")
  },
  abort(reason) {
    print("Aborted " + reason)
  }
});


var idx = 0;

const readStream = new ReadableStream({
  pull(ctrl) {
    console.log("Pulling", idx);
    switch (idx) {
      case 0:
        ctrl.enqueue("Hello");
        break;
      case 1:
        ctrl.enqueue("World")
      default:
        ctrl.close();
    }
    idx++;
  }
})

console.time('pipe')
await readStream.pipeTo(writeStream);
console.timeEnd("pipe")
print('output ' + output.join(" "))


const stream = ReadableStream.from([1,2,3,4]);

for await(const next of stream) {

  console.log('next',next, stream.locked)
}

console.log(stream.locked)


// let reader = readStream.getReader();

// while (true) {
//   const chunk = await reader.read();
//   if (chunk.done) {
//     break;
//   }
//   print("Read: " + chunk.value);
//   // output.push(chunk.value);
// }
// print("done")