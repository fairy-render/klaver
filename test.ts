/// <reference path="module.d.ts" />

import { cwd, args } from "@klaver/env";
import * as sh from "@klaver/shell";
import { EXPORT } from "./import.ts";
import { Client, Request, Headers, Cancel } from "@klaver/http";
import { delay } from "@klaver/base";
const client = new Client();
const req = new Request("https://distrowatch.com/");
// const out = await client.send(new Request("https://distrowatch.com/"));

setTimeout(() => {
	console.log("raprap", req);
	throw "Errored";
}, 100);

// console.log(await out.text(), EXPORT);
// async function test() {
// 	const { render } = await import("./server/entry-server.js");
// 	try {
// 		const output = await render();
// 		console.log("html", output);
// 	} catch (e) {
// 		console.log(e.message);
// 		print(e.stack);
// 	}
// }

// await test();

const o = await sh.sh("ls").output();

const decoder = new TextDecoder();

console.log(o);

await delay(1000);

console.log("Test mig", { i: 2 });

// const out = await sh.sh("docker", "image", "list", "--format", "json").output();

// print(JSON.stringify(JSON.parse(out), 2, 2));

// const client = new Client();

// const headers = new Headers();

// const cancel = new Cancel();

// const req = new Request("https://github.com", {
// 	method: "GET",
// 	headers,
// 	cancel,
// });

// setTimeout(() => {
// 	cancel.cancel();
// }, 1000);

// print(req.method);

// try {
// 	const resp = await client.send(req);

// 	print("Status " + resp.status);
// } catch (e) {
// 	print("error: " + e);
// }
