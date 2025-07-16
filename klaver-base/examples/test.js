import { CountQueuingStrategy, WritableStream, ReadableStream, WritableStreamDefaultController } from 'quick:base'

const output = []


// const test = new WritableStreamDefaultController({

// })

print("rapra")

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


await readStream.pipeTo(writeStream);

print('output ' + output.join(" "))