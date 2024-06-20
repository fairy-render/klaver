/// <reference path="module.d.ts" />

import { cwd, args } from "@klaver/env";
import * as sh from "@klaver/shell";
import { EXPORT } from "./import.ts";
import { Client, Request, Headers, Cancel } from "@klaver/http";

const client = new Client();

const out = await client.send(new Request("https://distrowatch.com/"));

console.log(await out.text(), EXPORT);
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
