
const channel = new MessageChannel

channel.port1.addEventListener("message", handleMessage, false);
function handleMessage(e) {
  console.log(e)
}

channel.port1.start();