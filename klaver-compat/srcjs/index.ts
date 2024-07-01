/// <reference path="../../module.d.ts" />

import { writeProps } from "./util.js";
import console from "./console.js";

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

	const { TextEncoder, TextDecoder } = await import("@klaver/encoding");

	writeProps(global, { TextDecoder, TextEncoder });

	console(global);
}
