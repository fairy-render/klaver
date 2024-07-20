declare module "@klaver/base" {
	export type TimerId = number;
	export function set_timeout(fn: () => void, ns?: number): TimerId;
	export function clear_timeout(id: TimerId): void;
	export function set_interval(fn: () => void, ns?: number): TimerId;
	export function clear_interval(id: TimerId);

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
}

declare module "@klaver/shell" {
	class Exec {
		output(): Promise<ArrayBuffer>;
		pipe(exec: Exec): Pipe;
	}

	class Pipe {
		output(): Promise<ArrayBuffer>;
		pipe(exec: Exec): Pipe;
	}

	export function cat(path: string): Promise<AsyncIterable<ArrayBuffer>>;
	export function sh(cmd: string, ...rest: string[]): Exec;
}

declare module "@klaver/env" {
	export function cwd(): string;
	export function args(): string[];
}

declare module "@klaver/http" {
	export class Client {
		constructor();

		get(url: string): Promise<Response>;

		send(req: Request): Promise<Response>;
	}

	export class Cancel {
		cancel(): void;
	}

	export type Method =
		| "GET"
		| "POST"
		| "PUT"
		| "PATCH"
		| "DELETE"
		| "HEAD"
		| "OPTION";

	class Headers {
		append(key: string, value: string): void;
		get(key: string): string;
		has(key: string): boolean;
	}

	export interface RequestInit {
		body?: ArrayBuffer;
		method?: Method;
		headers?: Headers | Record<string, string>;
		cancel?: Cancel;
	}

	export class Request {
		readonly url: string;
		readonly method: Method;
		constructor(url: string, opts?: RequestInit);

		text(): Promise<string>;
		json<T = unknown>(): Promise<T>;
		stream(): AsyncIterable<ArrayBuffer>;
	}

	export type HeadersInit =
		| [string, string][]
		| Record<string, string>
		| Headers;

	export interface ResponseInit {
		status?: number;
		headers?: HeadersInit;
	}

	export class Response {
		readonly url: string;
		readonly status: number;
		readonly headers: Headers;

		constructor(body?: ArrayBuffer, options?: ResponseInit);

		text(): Promise<string>;
		json<T = unknown>(): Promise<T>;
		stream(): AsyncIterable<ArrayBuffer>;
	}

	export function createCancel(): Cancel;
}

declare module "@klaver/encoding" {
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
}

declare module "@klaver/crypto" {
	export type Buffer =
		| ArrayBuffer
		| Uint8Array
		| Int8Array
		| Uint16Array
		| Int16Array
		| Uint32Array
		| Int32Array;

	export function randomUUID(): string;
	export function getRandomValues(
		buffer:
			| Uint8Array
			| Int8Array
			| Uint16Array
			| Int16Array
			| Uint32Array
			| Int32Array,
	): void;

	export type Algo = "sha1" | "sha256";

	export class Digest {
		constructor(algo: Algo);

		update(data: Buffer): void;
		digest(): ArrayBuffer;
	}
}

declare function print(...args: unknown[]): void;

declare type TimeId = number;

declare interface Timers {
	createTimer(cb: () => void, delay: number, repeat?: boolean): TimeId;
	clearTimer(id: TimeId): void;
}

declare interface FormatOptions {
	colors: boolean;
}

declare interface Core {
	readonly timers: Timers;
	readonly format: (value: unknown, options?: FormatOptions) => string;
}

declare const Core: Core;
