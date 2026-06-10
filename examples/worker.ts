

globalThis.onmessage = (event) => {
    console.log('From server:', event.data);
}


await fetch("https://jsonplaceholder.typicode.com/todos/1")

globalThis.postMessage('Hello from worker');

