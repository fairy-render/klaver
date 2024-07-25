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

export class Headers {
	append(key: string, value: string): void;
	get(key: string): string;
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
	stream(): AsyncIterable<ArrayBuffer>;
}

export class Url {
	constructor(url: string, base?: string);

	readonly href: string;
	readonly port: string;
	readonly hash: string;
	readonly password: string;
	readonly protocol: string;
	readonly search: string;
}

export function createCancel(): Cancel;
