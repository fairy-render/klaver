((global) => {
  
  global.__triggerMessage = (msg) => {
    if (typeof global.onmessage === 'function') {
      global.onmessage(msg);
    }
  }

})(globalThis)