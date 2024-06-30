const { Client, Request } = await import("@klaver/http");

const client = new Client();

const resp = await client.send(new Request("https://google.com"));

try {
	print(await resp.text());
} catch (e) {
	print(e.message ?? "");
}

function delay(ns) {
	return new Promise((res) => setTimeout(res, ns));
}

setTimeout(() => {
	print("After after");
}, 2000);

print("Hello, World");
await delay(1000);
print("after");
