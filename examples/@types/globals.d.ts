declare global {
 type Buffer =
	| ArrayBuffer
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

 type TypedArray =
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

 type TimerId = unknown;

 function setTimeout(callback: () => void, timeout?: number): unknown;

 class Event {}

 class EventTarget {
	constructor();

	addEventListener(event: string, callback: (event: Event) => void): void;
}

 class AbortController {
	constructor();

	readonly signal: AbortSignal;

	abort(): void;
}

 class AbortSignal extends EventTarget {}

interface ConsoleApi {
	log(...args: unknown[]): void;
}

interface PerformanceApi {
	now(): number;
	timeOrigin: number;
}

 class TextEncoder {
	constructor(label?: string);

	readonly encoding: string;
	encode(input: string): Uint8Array;
}

 class TextDecoder {
	constructor(label?: string);

	readonly encoding: string;
	decode(input: ArrayBuffer): string;
}

 function atob(input: string): string;
 function btoa(input: string): string;

 class Response {}



 interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer);
}

 interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

 const crypto: Crypto;
}