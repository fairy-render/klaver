/// <reference path="../../module.d.ts" />

import { writeProps } from "./util.js";
import fetch from "./fetch/index.js";
import stream from "./streams.js";
import process from "./process.js";
import crypto from "./crypto.js";
import base from "./base.js";

// const Core = globalThis.Core;

export default async function main(global: Record<string, unknown>) {
	writeProps(global, {
		setTimeout: (cb: () => void, timeout?: number) => {
			return Core.timers.createTimer(cb, timeout, false);
		},
		clearTimeout: Core.timers.clearTimer.bind(Core.timers),
		setInterval: (cb: () => void, timeout?: number) => {
			return Core.timers.createTimer(cb, timeout, true);
		},
		clearInterval: Core.timers.clearTimer.bind(Core.timers),
	});

	const { TextEncoder, TextDecoder, btoa, atob } = await import(
		"@klaver/encoding"
	);

	writeProps(global, { TextDecoder, TextEncoder, btoa, atob });

	await base(global);

	// console(global);
	stream(global);
	await crypto(global);
	await fetch(global);
	process(global);
}
