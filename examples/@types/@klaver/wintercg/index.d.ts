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
	debug(...args: unknown[]): void;
	warn(...args: unknown[]): void;
	error(...args: unknown[]): void;
}

interface PerformanceApi {
	now(): number;
	timeOrigin: number;
}

// Streams

export interface UnderlyingSource {
	pull(): Promise<void>;
}

export class ReadableStream {
	constructor(source: UnderlyingSource);
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

export type Method =
	| "GET"
	| "POST"
	| "PUT"
	| "PATCH"
	| "DELETE"
	| "HEAD"
	| "OPTION";

export class Headers {
	append(key: string, value: string): void;
	set(key: string, value: string): void;
	get(key: string): string;
	getAll(key: string): string[];
	has(key: string): boolean;
}

export type Body =
	| ArrayBuffer
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Int32Array
	| Uint32Array
	| string;

export interface RequestInit {
	body?: Body;
	method?: Method;
	headers?: HeadersInit;
	signal?: AbortSignal;
}

export class Request {
	constructor(url: string | URL, opts?: RequestInit);

	readonly url: URL;
	readonly method: Method;

	text(): Promise<string>;
	json<T = unknown>(): Promise<T>;
	readonly body: ReadableStream;
}

export type HeadersInit = [string, string][] | Record<string, string> | Headers;

export interface ResponseInit {
	status?: number;
	headers?: HeadersInit;
}

export class Response {
	readonly url: string;
	readonly status: number;
	readonly headers: Headers;

	constructor(body?: Body, options?: ResponseInit);

	text(): Promise<string>;
	json<T = unknown>(): Promise<T>;
	arrayBuffer(): Promise<ArrayBuffer>;
	stream(): AsyncIterable<ArrayBuffer>;
}

export class URL {
	constructor(url: string | URL, base?: string | URL);

	href: string;
	port: string;
	hash: string;
	password: string;
	protocol: string;
	search: string;
}



export interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer);
}

export interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

export const crypto: Crypto;
