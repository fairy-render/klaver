
const worker = new Worker("./klaver/examples/worrker.js");

worker.addEventListener('message', (e) => {
  console.log('callback', e)
});

worker.postMessage("Hello from parent")