/// <reference path="module.d.ts" />
/* jsxSourceImport @wilbur/template */
import { EXPORT } from "./import.ts";

const i = setInterval(() => {
  print("interval");
}, 450);

print(i);

setTimeout(() => {
  console.log("raprap");
}, 10);

setTimeout(() => {
  console.log("Hello");
  clearInterval(i);
}, 1000);

print("Hello, World!", 2020, false, EXPORT);

const abort: AbortController = new AbortController();

const resp = await fetch("https://google.com", { signal: abort.signal }).then(
  (resp) => resp.text()
);

console.log("test", resp);

const test = () => {
  return <div>Test</div>;
};
