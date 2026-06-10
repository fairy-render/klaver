declare const Fs:FileSystem;

/// <reference no-default-lib="true"/>
/// <reference lib="es2021" />
/// <reference lib="es2022.array" />
/// <reference lib="es2022.error" />
/// <reference lib="es2022.object" />
/// <reference lib="es2022.sharedmemory" />
/// <reference lib="es2022.string" />


declare interface ImportMeta {
    url: string;
}


declare type Buffer =
    | ArrayBuffer
    | Uint8Array
    | Int8Array
    | Uint16Array
    | Int16Array
    | Uint32Array
    | Int32Array;

declare type TypedArray =
    | Uint8Array
    | Int8Array
    | Uint16Array
    | Int16Array
    | Uint32Array
    | Int32Array;

// EventTarget

interface EventListener {
    (evt: Event): void;
}

interface EventListenerObject {
    handleEvent(object: Event): void;
}

type EventListenerOrEventListenerObject = EventListener | EventListenerObject;

interface EventSourceEventMap {
    "error": Event;
}

interface Event { }

declare var Event: {
    prototype: Event,
    new(type: String): Event
};

interface EventTarget {
    addEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
    removeEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
    dispatchEvent(event: Event): boolean;
}

declare var EventTarget: {
    prototype: EventTarget;
    new(): EventTarget;
}


// Console

interface Console {
    log(...data: any[]): void;
    warn(...data: any[]): void;
    error(...data: any[]): void;
    info(...data: any[]): void;
    debug(...data: any[]): void;
    time(label?: string): void;
    timeEnd(label?: string): void;
}

declare var console: Console;


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




declare interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer): void;
}

declare interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

declare const crypto: Crypto;

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




declare var Worker: {
    new(scriptURL: string): WorkerInstance;
    prototype: WorkerInstance;
}

declare interface WorkerInstance {
    postMessage(message: any): void;
    onmessage: ((event: any) => void) | null;
    addEventListener(type: "message", listener: (event: any) => void): void;
    removeEventListener(type: "message", listener: (event: any) => void): void;
    terminate(): void;
}



declare interface File {
    read(len: number): Promise<ArrayBuffer>;
    arrayBuffer(): Promise<ArrayBuffer>;
    write(buffer: ArrayBuffer): Promise<void>;
}


declare interface FileSystem {
    readonly name: string;
    readonly root: FileSystemEntry
}

declare interface FileSystemEntry {
    readonly fileName: string;
    readonly extension: string;

    toString(): string;
    resolve(path: string): FileSystemEntry;
    metadata(): Promise<Metadata>;
    listDir(): Promise<IterableIterator<FileSystemEntry>>;
    open(opts: OpenOptions): Promise<File>
}

declare interface Metadata {
    size: number;
    type: 'dir' | 'file'
}

declare interface OpenOptions {
    read?: boolean;
    write?: boolean;
    append?: boolean;
    create?: boolean;
    truncate?: boolean;
}