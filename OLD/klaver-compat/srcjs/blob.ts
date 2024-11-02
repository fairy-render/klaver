export type BlobPart = string | ArrayBuffer | Blob;

export class Blob {
	constructor(parts?: BlobPart[]) {}
}
