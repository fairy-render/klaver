export interface ConsoleApi {
	log(...args: unknown[]): void;
	warn(...args: unknown[]): void;
}

export class Console implements ConsoleApi {
	#target: string;
	constructor(target = "") {
		this.#target = target;
	}

	log(...args: unknown[]): void {
		this.#print("info", args);
	}
	warn(...args: unknown[]): void {
		this.#print("warn", args);
	}
	error(...args: unknown[]): void {
		this.#print("warn", args);
	}

	#print(level: string, args: unknown[]) {
		const formatted = args.map((m) => format(m)).join(", ");
		print(`[${level}] ${formatted}`);
	}
}

export function log(...args: unknown[]) {
	const formatted = args.map((m) => format(m)).join(", ");
	print(`${formatted}`);
}

export default function init(global: Record<string, unknown>) {
	Object.defineProperties(global, {
		Console: { value: Console },
		console: { value: new Console() },
	});
}

function format(value: unknown, embed = false): string {
	if (Array.isArray(value)) {
		return formatArray(value);
	}

	switch (typeof value) {
		case "string":
			return embed ? `"${value}"` : value;
		case "number":
		case "undefined":
			return `${value}`;
		case "function":
			return `<Function ${value.name}>`;
		case "object":
			return formatObject(value);
	}
}

function formatObject(value: object) {
	let output = "{";
	let count = 0;
	for (const key in value) {
		if (count++ > 0) {
			output += ",";
		}
		output += ` ${format(key, true)}: ${format(value[key], true)}`;
	}

	output += " }";

	return output;
}

function formatArray(value: unknown[]) {
	return `[${value.map((m) => format(m, true)).join(", ")}]`;
}
