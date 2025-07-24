
// setTimeout(() => {
//   console.log("Hello");
// }, 0)

const ctrl = new AbortController();

setTimeout(()=> {
  console.log('dsdsdpsdp')
}
, 10);

const test = await fetch("https://jsonplaceholder.typicode.com/posts/1", {
})
  .then(response => {
    console.log("Response received", response);
    return response.text()
  })



console.log("Klaver.js example", test);