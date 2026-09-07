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

declare function queueMicrotask(callback: () => void): void;

declare var self: typeof globalThis;


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


// Performance

type DOMHighResTimeStamp = number;

interface Performance extends EventTarget {
    readonly timeOrigin: DOMHighResTimeStamp;
    now(): DOMHighResTimeStamp;
    toJSON(): any;
}

declare var Performance: {
    prototype: Performance;
};

declare var performance: Performance;


// Console

interface Console {
    log(...data: any[]): void;
    warn(...data: any[]): void;
    error(...data: any[]): void;
    info(...data: any[]): void;
    debug(...data: any[]): void;
    assert(condition?: boolean, ...data: any[]): void;
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

declare class TextEncoderStream {
    readonly encoding: string;
    readonly readable: ReadableStream;
    readonly writable: WritableStream;
}

declare class TextDecoderStream {
    constructor(label?: string);

    readonly encoding: string;
    readonly readable: ReadableStream;
    readonly writable: WritableStream;
}

declare function atob(input: string): string;
declare function btoa(input: string): string;


// Blob

type BlobPart = Blob | Buffer | string;

interface BlobPropertyBag {
    type?: string;
}

interface Blob {
    readonly size: number;
    readonly type: string;
    arrayBuffer(): Promise<ArrayBuffer>;
    bytes(): Promise<Uint8Array>;
    slice(start?: number, end?: number, contentType?: string): Blob;
    stream(): ReadableStream;
    text(): Promise<string>;
}

declare var Blob: {
    prototype: Blob;
    new(blobParts?: BlobPart[], options?: BlobPropertyBag): Blob;
};

// File

interface FilePropertyBag extends BlobPropertyBag {
    lastModified?: number;
}

interface File extends Blob {
    readonly name: string;
    readonly lastModified: number;
}

declare var File: {
    prototype: File;
    new(fileBits: BlobPart[], fileName: string, options?: FilePropertyBag): File;
};
