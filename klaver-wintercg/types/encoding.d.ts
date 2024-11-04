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

export function atob(input: string): string;
export function btoa(input: string): string;
