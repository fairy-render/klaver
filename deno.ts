export {};

function sleep(ms: number = 0) {
  return new Promise((res) => setTimeout(res, ms));
}

await sleep(500);

console.log(
  "test",
  await fetch("https://loppen.dk/", {
    headers: {
      "User-Agent":
        "Mozilla/5.0 (X11; Linux x86_64; rv:133.0) Gecko/20100101 Firefox/133.0",
    },
  }).then((m) => m.text())
);

// console.log('raprap')
