import { AbortController } from "@klaver/base";

const emitter = new EventTarget();

emitter.addEventListener("build", (event) => {
	console.log(event.type);
});

emitter.dispatchEvent(new Event("build"));

const ctr = new AbortController();

ctr.signal.onabort = () => {
	console.log("aboty");
};

ctr.abort();

const blob = new Blob(["Hello", "World"]);

console.log(Object.getOwnPropertyDescriptors(blob));

console.log(blob.size);

console.log(await blob.text());
