console.log("Hello, Klaver!");

const n = setInterval(() => {
  console.log("This is an interval message!");
}, 500);

setTimeout(() => {
  console.log("This is a timeout message!");
  clearInterval(n);
}, 2001);
