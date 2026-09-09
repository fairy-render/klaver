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




declare interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer): void;
}

type KeyUsage =
  | "encrypt"
  | "decrypt"
  | "sign"
  | "verify"
  | "deriveKey"
  | "deriveBits"
  | "wrapKey"
  | "unwrapKey";

type KeyType = "secret" | "private" | "public";
type KeyFormat = "raw" | "pkcs8" | "spki" | "jwk";

type AlgorithmIdentifier = string | { name: string;[key: string]: unknown };
type BufferSource = ArrayBuffer | ArrayBufferView;

interface KeyAlgorithm {
  name: string;
}

interface AesKeyAlgorithm extends KeyAlgorithm {
  length: number;
}

interface HmacKeyAlgorithm extends KeyAlgorithm {
  hash: KeyAlgorithm;
  length: number;
}

interface RsaHashedKeyAlgorithm extends KeyAlgorithm {
  modulusLength: number;
  publicExponent: Uint8Array;
  hash: KeyAlgorithm;
}

interface EcKeyAlgorithm extends KeyAlgorithm {
  namedCurve: "P-256" | "P-384" | "P-521";
}

interface CryptoKey {
  readonly type: KeyType;
  readonly extractable: boolean;
  readonly algorithm:
  | AesKeyAlgorithm
  | HmacKeyAlgorithm
  | RsaHashedKeyAlgorithm
  | EcKeyAlgorithm
  | KeyAlgorithm;
  readonly usages: KeyUsage[];
}

declare var CryptoKey: {
  prototype: CryptoKey;
};

interface CryptoKeyPair {
  publicKey: CryptoKey;
  privateKey: CryptoKey;
}

interface JsonWebKey {
  kty: string;
  k?: string;
  alg?: string;
  ext?: boolean;
  key_ops?: string[];
  // RSA
  n?: string;
  e?: string;
  d?: string;
  p?: string;
  q?: string;
  dp?: string;
  dq?: string;
  qi?: string;
  // EC
  crv?: string;
  x?: string;
  y?: string;
}

interface AesKeyGenParams {
  name: "AES-GCM" | "AES-CBC" | "AES-CTR";
  length: 128 | 192 | 256;
}

interface HmacKeyGenParams {
  name: "HMAC";
  hash: AlgorithmIdentifier;
  length?: number;
}

interface HmacImportParams {
  name: "HMAC";
  hash: AlgorithmIdentifier;
}

interface AesGcmParams {
  name: "AES-GCM";
  iv: BufferSource;
  additionalData?: BufferSource;
  /** Bits. Only the default, 128, is currently supported. */
  tagLength?: number;
}

interface AesCbcParams {
  name: "AES-CBC";
  iv: BufferSource;
}

interface AesCtrParams {
  name: "AES-CTR";
  counter: BufferSource;
  /** Bits (1-128) of `counter` that actually form the wrapping counter. */
  length: number;
}

interface RsaHashedKeyGenParams {
  name: "RSASSA-PKCS1-v1_5" | "RSA-OAEP" | "RSA-PSS";
  modulusLength: number;
  /** Big-endian bytes, e.g. `new Uint8Array([1, 0, 1])` for 65537. */
  publicExponent: Uint8Array;
  hash: AlgorithmIdentifier;
}

interface RsaHashedImportParams {
  name: "RSASSA-PKCS1-v1_5" | "RSA-OAEP" | "RSA-PSS";
  hash: AlgorithmIdentifier;
}

interface RsaOaepParams {
  name: "RSA-OAEP";
  label?: BufferSource;
}

interface RsaPssParams {
  name: "RSA-PSS";
  /** Bytes. */
  saltLength: number;
}

interface EcKeyGenParams {
  name: "ECDSA" | "ECDH";
  namedCurve: "P-256" | "P-384" | "P-521";
}

interface EcKeyImportParams {
  name: "ECDSA" | "ECDH";
  namedCurve: "P-256" | "P-384" | "P-521";
}

interface EcdsaParams {
  name: "ECDSA";
  hash: AlgorithmIdentifier;
}

interface EcdhKeyDeriveParams {
  name: "ECDH";
  public: CryptoKey;
}

interface HkdfParams {
  name: "HKDF";
  hash: AlgorithmIdentifier;
  salt: BufferSource;
  info: BufferSource;
}

interface HkdfImportParams {
  name: "HKDF";
}

interface Pbkdf2Params {
  name: "PBKDF2";
  hash: AlgorithmIdentifier;
  salt: BufferSource;
  iterations: number;
}

interface Pbkdf2ImportParams {
  name: "PBKDF2";
}

declare interface SubtleCrypto {
  digest(
    algo: "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512",
    input: BufferSource,
  ): Promise<ArrayBuffer>;

  encrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;
  decrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  sign(
    algorithm: AlgorithmIdentifier | EcdsaParams | RsaPssParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;
  verify(
    algorithm: AlgorithmIdentifier | EcdsaParams | RsaPssParams,
    key: CryptoKey,
    signature: BufferSource,
    data: BufferSource,
  ): Promise<boolean>;

  generateKey(
    algorithm: AesKeyGenParams | HmacKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  generateKey(
    algorithm: RsaHashedKeyGenParams | EcKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKeyPair>;
  importKey(
    format: KeyFormat,
    keyData: BufferSource | JsonWebKey,
    algorithm:
    | "AES-GCM"
    | "AES-CBC"
    | "AES-CTR"
    | HmacImportParams
    | RsaHashedImportParams
    | EcKeyImportParams
    | HkdfImportParams
    | Pbkdf2ImportParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  exportKey(format: KeyFormat, key: CryptoKey): Promise<ArrayBuffer | JsonWebKey>;

  deriveBits(
    algorithm: EcdhKeyDeriveParams | HkdfParams | Pbkdf2Params,
    baseKey: CryptoKey,
    length?: number | null,
  ): Promise<ArrayBuffer>;
  deriveKey(
    algorithm: EcdhKeyDeriveParams | HkdfParams | Pbkdf2Params,
    baseKey: CryptoKey,
    derivedKeyAlgorithm: AesKeyGenParams | HmacKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;

  wrapKey(
    format: KeyFormat,
    key: CryptoKey,
    wrappingKey: CryptoKey,
    wrapAlgorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
  ): Promise<ArrayBuffer>;
  unwrapKey(
    format: KeyFormat,
    wrappedKey: BufferSource,
    unwrappingKey: CryptoKey,
    unwrapAlgorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    unwrappedKeyAlgorithm:
    | "AES-GCM"
    | "AES-CBC"
    | "AES-CTR"
    | HmacImportParams
    | RsaHashedImportParams
    | EcKeyImportParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
}

declare const crypto: Crypto;


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