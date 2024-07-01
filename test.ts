/// <reference path="module.d.ts" />

import { EXPORT } from "./import.ts";

const i = setInterval(() => {
	print("interval");
}, 450);

print(i);

setTimeout(() => {
	console.log("Hello");
	clearInterval(i);
}, 1000);

print("Hello, World!", 2020, false, EXPORT);
