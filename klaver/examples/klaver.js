
// setTimeout(() => {
//   console.log("Hello");
// }, 0)

const ctrl = new AbortController();




// setTimeout(() => {
//   console.log('dsdsdpsdp')
//   ctrl.abort();
// }
//   , 10);

const url = new URL("https://jsonplaceholder.typicode.com/posts/1")

const test = await fetch(url, {
  signal: ctrl.signal,
})
  .then(response => {
    console.log("Response received", response);
    return response.json()
  })



console.log("Klaver.js example", test);