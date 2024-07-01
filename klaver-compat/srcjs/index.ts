/// <reference path="../../module.d.ts" />

function writeProp(out: Record<string, unknown>, name: string, value: unknown) {
	Object.defineProperty(out, name, {
		writable: true,
		configurable: true,
		enumerable: true,
		value,
	});
}

const Core = globalThis.Core;

export default async function main(global: Record<string, unknown>) {
	writeProp(global, "setTimeout", (cb: () => void, timeout?: number) => {
		return Core.timers.createTimer(cb, timeout, false);
	});
	writeProp(global, "clearTimeout", Core.timers.clearTimer.bind(Core.timers));
	writeProp(global, "setInterval", (cb: () => void, timeout?: number) => {
		return Core.timers.createTimer(cb, timeout, true);
	});
	writeProp(global, "clearInterval", Core.timers.clearTimer.bind(Core.timers));

	const { TextEncoder, TextDecoder } = await import("@klaver/encoding");

	writeProp(global, "TextEncoder", TextEncoder);
	writeProp(global, "TextDecoder", TextDecoder);
}
