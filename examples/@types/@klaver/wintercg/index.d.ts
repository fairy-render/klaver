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

export class TextEncoder {
	constructor(label?: string);

	readonly encoding: string;
	encode(input: string): Uint8Array;
}

export class TextDecoder {
	constructor(label?: string);

	readonly encoding: string;
	decode(input: ArrayBuffer): string;
}

export function atob(input: string): string;
export function btoa(input: string): string;

export class Response {}



export interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer);
}

export interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

export const crypto: Crypto;
