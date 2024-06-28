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
		append(key: string, value: string);
	}

	export interface RequestInit {
		body?: any;
		method?: Method;
		headers?: Headers | Record<string, string>;
		cancel?: Cancel;
	}

	export class Request {
		readonly url: string;
		readonly method: Method;
		constructor(url: string, opts?: RequestInit);
	}

	export class Response {
		readonly url: string;
		readonly status: number;

		text(): Promise<string>;
		stream(): Promise<AsyncIterable<ArrayBuffer>>;
	}

	export function createCancel(): Cancel;
}

declare function print(...args: unknown[]);
