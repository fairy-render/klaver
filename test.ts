/// <reference path="module.d.ts" />

const i = setInterval(() => {
	print("interval");
}, 0);

print(i);

clearInterval(i);

setTimeout(() => {
	print("Hello");
	clearInterval(i);
}, 1000);

print("Hello, World!", 2020, false);
