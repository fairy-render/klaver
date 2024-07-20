/// <reference no-default-lib="true"/>

/// <reference lib="es2021" />
/// <reference lib="es2022.array" />
/// <reference lib="es2022.error" />
/// <reference lib="es2022.object" />
/// <reference lib="es2022.sharedmemory" />
/// <reference lib="es2022.string" />

import type { Buffer } from "@klaver/crypto";

declare global {
	interface Crypto {
		readonly subtle: SubtleCrypto;
		randomUUID(): string;
		getRandomValues(buffer: Buffer);
	}

	interface SubtleCrypto {
		digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
	}

	const crypto: Crypto;

	interface ConsoleApi {
		log(...args: unknown[]): void;
	}

	const console: ConsoleApi;

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

	function atob(input: string): string;
	function btoa(input: string): string;

	class Request {
		constructor(input: Request | string);
	}

	class Headers {
		get(name: string): string | undefined;
	}

	type HeadersInit = [string, string][] | Record<string, string> | Headers;

	abstract class Body {
		arrayBuffer(): Promise<ArrayBuffer>;
	}

	interface ResponseOptions {
		headers?: HeadersInit;
		status?: number;
	}

	class Response extends Body {
		constructor(body?: any, options?: ResponseOptions);

		readonly headers: Headers;
	}
}
