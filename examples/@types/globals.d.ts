/// <reference no-default-lib="true"/>

/// <reference lib="es2021" />
/// <reference lib="es2022.array" />
/// <reference lib="es2022.error" />
/// <reference lib="es2022.object" />
/// <reference lib="es2022.sharedmemory" />
/// <reference lib="es2022.string" />

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

declare function setTimeout(callback: () => void, timeout?: number): unknown;

declare class Event {}

declare class EventTarget {
	constructor();

	addEventListener(event: string, callback: (event: Event) => void): void;
}

declare class AbortController {
	constructor();

	readonly signal: AbortSignal;

	abort(): void;
}

declare class AbortSignal extends EventTarget {}

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

 interface UnderlyingSource {
	pull(): Promise<void>;
}

declare class ReadableStream {
	constructor(source: UnderlyingSource);
}

declare class TextEncoder {
	constructor(label?: string);

	readonly encoding: string;
	encode(input: string): Uint8Array;
}

declare class TextDecoder {
	constructor(label?: string);

	readonly encoding: string;
	decode(input: ArrayBuffer): string;
}

declare function atob(input: string): string;
declare function btoa(input: string): string;

 type Method =
	| "GET"
	| "POST"
	| "PUT"
	| "PATCH"
	| "DELETE"
	| "HEAD"
	| "OPTION";

declare class Headers {
	append(key: string, value: string): void;
	set(key: string, value: string): void;
	get(key: string): string;
	getAll(key: string): string[];
	has(key: string): boolean;
}

 type Body =
	| ArrayBuffer
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Int32Array
	| Uint32Array
	| string;

 interface RequestInit {
	body?: Body;
	method?: Method;
	headers?: HeadersInit;
	signal?: AbortSignal;
}

declare class Request {
	constructor(url: string | URL, opts?: RequestInit);

	readonly url: URL;
	readonly method: Method;

	text(): Promise<string>;
	json<T = unknown>(): Promise<T>;
	readonly body: ReadableStream;
}

 type HeadersInit = [string, string][] | Record<string, string> | Headers;

 interface ResponseInit {
	status?: number;
	headers?: HeadersInit;
}

declare class Response {
	readonly url: string;
	readonly status: number;
	readonly headers: Headers;

	constructor(body?: Body, options?: ResponseInit);

	text(): Promise<string>;
	json<T = unknown>(): Promise<T>;
	arrayBuffer(): Promise<ArrayBuffer>;
	stream(): AsyncIterable<ArrayBuffer>;
}

declare class URL {
	constructor(url: string | URL, base?: string | URL);

	href: string;
	port: string;
	hash: string;
	password: string;
	protocol: string;
	search: string;
}



 interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer);
}

 interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

declare const crypto: Crypto;
// Globals

declare const console: ConsoleApi;
declare const performance: PerformanceApi;
