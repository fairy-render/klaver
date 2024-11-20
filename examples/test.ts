import { open } from "@klaver/fs";
import { Handlebars } from "@klaver/hbs";
import { hello } from "./other.ts";

const hbs = new Handlebars();

hbs.registerTemplate("main", "{{name}}, World");

const output = hbs.render("main", { name: "Hello" });

console.log(output);

console.log(process.env.PATH);
process.env.RAPPER = "Rasmus";

console.log(Object.keys(process.env));

hello();
// console.log(btoa(atob(new TextDecoder().decode(await resp.arrayBuffer()))));

// console.log(resp.headers.get("Content-Type"));

// // console.log(new Response(test.buffer).body);

// const ints = new Int16Array(10);

// crypto.getRandomValues(ints);

// console.log(crypto.randomUUID());

// for (let i = 0; i < 10; i++) {
// 	console.log("Random number", ints[i]);
// }

// async function digestMessage(message) {
// 	const msgUint8 = new TextEncoder().encode(message); // encode as (utf-8) Uint8Array
// 	const hashBuffer = await crypto.subtle.digest("SHA-1", msgUint8); // hash the message
// 	const hashArray = Array.from(new Uint8Array(hashBuffer)); // convert buffer to byte array
// 	const hashHex = hashArray
// 		.map((b) => b.toString(16).padStart(2, "0"))
// 		.join(""); // convert bytes to hex string
// 	return hashHex;
// }

// console.log(await digestMessage("Message"));

// throw new Error("Hello");
