export default async function init(global: Record<string, unknown>) {
	const {
		TextDecoder,
		TextEncoder,
		setInterval,
		setTimeout,
		clearInterval,
		clearTimeout,
	} = await import("@klaver/base");

	print("typeof " + (await import("@klaver/base")).setInterval);

	Object.defineProperties(global, {
		TextDecoder: {
			value: TextDecoder,
		},
		TextEncoder: {
			value: TextEncoder,
		},
		setTimeout: {
			value: setTimeout,
		},
	});

	// Object.assign(global, {
	// 	TextDecoder,
	// 	TextEncoder,
	// 	setInterval,
	// 	setTimeout,
	// 	clearInterval,
	// 	clearTimeout,
	// });
}
