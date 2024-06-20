/// <reference path="../../module.d.ts" />
import initFetch from "./fetch/index.js";
import initStreams from "./streams.js";
import initConsole from "./console.js";
// import initBase from "./base.js";

async function init(global: Record<string, unknown>) {
	// await initBase(global);
	initConsole(global);
	initStreams(global);
	await initFetch(global);
}

await init(globalThis);
