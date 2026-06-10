(port, global) => {
  const event = new EventTarget();

  Object.defineProperty(global, "onmessage", {
    get() {
      return port.onmessage;
    },
    set(value) {
      port.onmessage = value;
    },
  });

  // port.addEventListener("message", (e) => {
  //   event.dispatchEvent(e);
  //   global.onmessage?.(e);
  // });

  global.postMessage = (msg) => {
    port.postMessage(msg);
  };

  global.addEventListener = (type, listener) => {
    port.addEventListener(type, listener);
  };

  global.removeEventListener = (type, listener) => {
    port.removeEventListener(type, listener);
  };
};
