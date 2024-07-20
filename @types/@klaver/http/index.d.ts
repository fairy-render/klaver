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

export type HeadersInit = [string, string][] | Record<string, string> | Headers;

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
