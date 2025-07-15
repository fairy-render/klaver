
export {}

const output = []


// const test = new WritableStreamDefaultController({

// })

console.log("rapra")



const stream = new WritableStream({
  start:()  => {
    console.log("Started")
  },
  write(chunk) {
    console.log('write')
    output.push(chunk)
  },
  close() {
    console.log("Close")
  }
});

const writer = stream.getWriter();





 writer.write("Hello").then(m => console.log('writting'));
 writer.write(",")
 writer.write(" World!")
 

await writer.close()

console.log("Her");

console.log('output ' + output.join(""))