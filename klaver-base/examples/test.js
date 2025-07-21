import { CountQueuingStrategy, WritableStream, ReadableStream, WritableStreamDefaultController, Console, EventTarget, AbortSignal, MessageChannel } from 'quick:base'

const output = []






const console = new Console((level, msg) => {
  print(`[${level}] ${msg}`)
})

console.log(new AbortSignal() instanceof EventTarget);

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