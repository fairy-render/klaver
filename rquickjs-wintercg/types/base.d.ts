export type Buffer =
	| ArrayBuffer
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

export type TypedArray =
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

export type TimerId = unknown;

export function setTimeout(callback: () => void, timeout?: number): unknown;

export class Event {}

export class EventTarget {
	constructor();

	addEventListener(event: string, callback: (event: Event) => void): void;
}

export class AbortController {
	constructor();

	readonly signal: AbortSignal;

	abort(): void;
}

export class AbortSignal extends EventTarget {}

interface ConsoleApi {
	log(...args: unknown[]): void;
}

interface PerformanceApi {
	now(): number;
	timeOrigin: number;
}
