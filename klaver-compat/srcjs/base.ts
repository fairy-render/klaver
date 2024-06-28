import { log } from "./console.js";

export default async function init(global: Record<string, unknown>) {
	const {
		TextDecoder,
		TextEncoder,
		set_interval,
		set_timeout,
		clear_interval,
		clear_timeout,
	} = await import("@klaver/base");

	Object.assign(global, {
		TextDecoder,
		TextEncoder,
		setInterval: set_interval,
		setTimeout: set_timeout,
		clearInterval: clear_interval,
		clearTimeout: clear_timeout,
	});
}
