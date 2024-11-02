console.log(typeof atob);

const test = new TextEncoder().encode("Hello");

console.log(test.buffer);

const resp = new Response(test, {
	headers: {
		"Content-Type": "text/html",
	},
});

console.log(resp.arrayBuffer());

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
