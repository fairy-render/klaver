declare type Method =
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

declare type Body =
    | ArrayBuffer
    | Uint8Array
    | Int8Array
    | Uint16Array
    | Int16Array
    | Int32Array
    | Uint32Array
    | string;

declare interface RequestInit {
    body?: Body;
    method?: Method;
    headers?: HeadersInit;
    signal?: AbortSignal;
}

declare class Request {
    constructor(url: string | URL, opts?: RequestInit);

    readonly url: string;
    readonly method: Method;
    readonly headers: Headers;

    text(): Promise<string>;
    json<T = unknown>(): Promise<T>;
    readonly body: ReadableStream;
}

declare type HeadersInit = [string, string][] | Record<string, string> | Headers;

declare interface ResponseInit {
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
    pathname: string;
}

declare function fetch(
    url: string | URL | Request,
    opts?: RequestInit,
): Promise<Response>;

declare class URLSearchParams {
    constructor(init: string | [string, string][]);
    get(key: string): string | undefined;
    has(key: string): boolean;
    getAll(key: string): string[];
    set(key: string, value: string): void;
    append(key: string, value: string): void;
    delete(key: string): void;
    entries(): IterableIterator<[string, string]>;
}
