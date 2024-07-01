export function writeProp(
	out: Record<string, unknown>,
	name: string,
	value: unknown,
) {
	Object.defineProperty(out, name, {
		writable: true,
		configurable: true,
		enumerable: true,
		value,
	});
}

export function writeProps(
	out: Record<string, unknown>,
	value: Record<string, unknown>,
) {
	for (const key in value) {
		writeProp(out, key, value[key]);
	}
}
