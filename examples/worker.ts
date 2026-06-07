

globalThis.onmessage = (event) => {
    console.log('From server:', event.data);
}




globalThis.postMessage('Hello from worker');

