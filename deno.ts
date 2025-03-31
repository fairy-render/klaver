// Deno.serve((_req) => {
//   return new Response("Hello, World!");
// });

setTimeout(() => {
  throw new Error("dsdsds");
}, 1000);

console.log("heja");
