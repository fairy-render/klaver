export type Buffer =
	| ArrayBuffer
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

export type TypedArray =
	| Uint8Array
	| Int8Array
	| Uint16Array
	| Int16Array
	| Uint32Array
	| Int32Array;

export function randomUUID(): string;
export function getRandomValues(buffer: TypedArray): void;

export type Algo = "sha1" | "sha256";

export class Digest {
	constructor(algo: Algo);

	update(data: Buffer): void;
	digest(): ArrayBuffer;
}
