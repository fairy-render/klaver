const resp = fetch("https://dummyjson.com/products?limit=5").then((resp) =>
  resp.json()
);

console.log(await resp);
