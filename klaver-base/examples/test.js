import { CountQueuingStrategy, WritableStream, WritableStreamDefaultController} from 'quick:base'

const output = []


// const test = new WritableStreamDefaultController({

// })

print("rapra")

const stream = new WritableStream({
  start:()  => {
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

const writer = stream.getWriter();


await writer.ready;


 writer.write("Hello");
 writer.write(",")
 writer.write(" World!")

 writer.abort("Just because");

// await writer.close()

print("He 2r");


print('output ' + output.join(""))