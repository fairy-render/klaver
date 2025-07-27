

globalThis.port.onmessage = (e) => {
  console.log(e)
  globalThis.port.postMessage("Hello back")
}