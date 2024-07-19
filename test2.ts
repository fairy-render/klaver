console.log(typeof atob);

const test = new TextEncoder().encode("Hello");

console.log(test.buffer);

const resp = new Response(test, {
	headers: {
		"Content-Type": "text/html",
	},
});

console.log(btoa(atob(new TextDecoder().decode(await resp.arrayBuffer()))));

console.log(resp.headers.get("Content-Type"));

// console.log(new Response(test.buffer).body);

export type {};
