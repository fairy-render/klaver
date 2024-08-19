import { writeProps } from "./util.js";

export default async function init(globals: Record<string, unknown>) {
	const { EventTarget, Event, AbortController, AbortSignal } = await import(
		"@klaver/base"
	);

	writeProps(globals, {
		EventTarget,
		Event,
		AbortController,
		AbortSignal,
	});
}
