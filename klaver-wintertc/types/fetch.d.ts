declare type Method =
    | "GET"
    | "POST"
    | "PUT"
    | "PATCH"
    | "DELETE"
    | "HEAD"
    | "OPTIONS"
    | (string & {});

declare type HeadersInit = [string, string][] | Record<string, string> | Headers;

declare class Headers {
    constructor(init?: HeadersInit);

    append(key: string, value: string): void;
    set(key: string, value: string): void;
    get(key: string): string | undefined;
    getAll(key: string): string[];
    has(key: string): boolean;
    delete(key: string): void;
    forEach(callback: (value: string, key: string) => void): void;
    entries(): IterableIterator<[string, string]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<string>;
    [Symbol.iterator](): IterableIterator<[string, string]>;
}

declare type Body =
    | ArrayBuffer
    | Uint8Array
    | Int8Array
    | Uint16Array
    | Int16Array
    | Int32Array
    | Uint32Array
    | string
    | URLSearchParams
    | FormData;

declare type FormDataEntryValue = string | File;

declare class FormData {
    constructor();

    append(name: string, value: string): void;
    append(name: string, value: Blob, filename?: string): void;
    set(name: string, value: string): void;
    set(name: string, value: Blob, filename?: string): void;
    get(name: string): FormDataEntryValue | undefined;
    getAll(name: string): FormDataEntryValue[];
    has(name: string): boolean;
    delete(name: string): void;
    forEach(
        callback: (value: FormDataEntryValue, key: string) => void,
    ): void;
    entries(): IterableIterator<[string, FormDataEntryValue]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<FormDataEntryValue>;
    [Symbol.iterator](): IterableIterator<[string, FormDataEntryValue]>;
}

declare interface RequestInit {
    body?: Body | null;
    method?: Method;
    headers?: HeadersInit;
    signal?: AbortSignal;
}

declare class Request {
    constructor(input: string | URL | Request, opts?: RequestInit);

    readonly url: string;
    readonly method: Method;
    readonly headers: Headers;
    readonly signal: AbortSignal | undefined;
    readonly bodyUsed: boolean;
    readonly body: ReadableStream | undefined;

    text(): Promise<string>;
    json<T = unknown>(): Promise<T>;
    arrayBuffer(): Promise<ArrayBuffer>;
    bytes(): Promise<Uint8Array>;
    blob(): Promise<Blob>;
    formData(): Promise<FormData>;
    /** Throws if the body has already been read (or is locked). */
    clone(): Request;
}

declare interface ResponseInit {
    status?: number;
    statusText?: string;
    headers?: HeadersInit;
}

declare class Response {
    constructor(body?: Body | null, options?: ResponseInit);

    readonly url: string;
    readonly redirected: boolean;
    readonly status: number;
    readonly ok: boolean;
    readonly statusText: string;
    readonly headers: Headers;
    readonly bodyUsed: boolean;
    readonly body: ReadableStream | undefined;

    text(): Promise<string>;
    json<T = unknown>(): Promise<T>;
    arrayBuffer(): Promise<ArrayBuffer>;
    bytes(): Promise<Uint8Array>;
    blob(): Promise<Blob>;
    formData(): Promise<FormData>;
    /** Throws if the body has already been read (or is locked). */
    clone(): Response;
}

declare class URL {
    constructor(url: string | URL, base?: string | URL);

    static canParse(url: string | URL, base?: string | URL): boolean;

    href: string;
    readonly origin: string;
    protocol: string;
    username: string;
    password: string;
    host: string;
    hostname: string;
    port: string;
    pathname: string;
    search: string;
    readonly searchParams: URLSearchParams;
    hash: string;

    toString(): string;
    toJSON(): string;
}

declare function fetch(
    url: string | URL | Request,
    opts?: RequestInit,
): Promise<Response>;

declare type URLSearchParamsInit =
    | string
    | [string, string][]
    | Record<string, string>
    | Iterable<[string, string]>;

declare class URLSearchParams {
    constructor(init?: URLSearchParamsInit);

    readonly size: number;

    get(key: string): string | undefined;
    has(key: string): boolean;
    getAll(key: string): string[];
    set(key: string, value: string): void;
    append(key: string, value: string): void;
    delete(key: string): void;
    forEach(callback: (value: string, key: string) => void): void;
    entries(): IterableIterator<[string, string]>;
    keys(): IterableIterator<string>;
    values(): IterableIterator<string>;
    toString(): string;
    [Symbol.iterator](): IterableIterator<[string, string]>;
}
