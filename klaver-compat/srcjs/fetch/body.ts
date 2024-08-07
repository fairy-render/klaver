import { TextDecoder, TextEncoder } from "@klaver/encoding";
import { TextDecoderStream } from "@stardazed/streams-text-encoding";
import { ReadableStream, type Transformer } from "web-streams-polyfill";

const DECODER = new TextDecoder();
const ENCODER = new TextEncoder();

export abstract class Body {
	#body: ReadableStream<ArrayBuffer>;
	#used = false;

	get body() {
		return this.#body;
	}

	get bodyUsed() {
		return this.#used;
	}

	constructor(body: BodyInit | AsyncIterable<ArrayBuffer>, length?: number) {
		if (body && body instanceof ReadableStream) {
			this.#body = body;
		} else if (body && body instanceof ArrayBuffer) {
			this.#body = new ReadableStream({
				async pull(controller) {
					controller.enqueue(body);
					controller.close();
				},
			});
		} else if (typeof body === "string") {
			this.#body = new ReadableStream({
				async pull(controller) {
					controller.enqueue(ENCODER.encode(body).buffer);
					controller.close();
				},
			});
		} else if (
			body &&
			(body instanceof Uint8Array ||
				body instanceof Uint16Array ||
				body instanceof Uint32Array)
		) {
			this.#body = new ReadableStream({
				async pull(controller) {
					controller.enqueue(body.buffer);
					controller.close();
				},
			});
		} else if (body?.[Symbol.asyncIterator]) {
			// const iter = body[Symbol.asyncIterator]();
			let stream: AsyncIterator<ArrayBuffer>;
			this.#body = new ReadableStream({
				async pull(controller) {
					const { done, value } = await stream.next();
					if (value) {
						controller.enqueue(value);
					}
					if (done) {
						controller.close();
					}
				},
				async start(controller) {
					stream = body[Symbol.asyncIterator]();
				},
			});
		} else if (body != null) {
			console.log(body);
			throw new TypeError(`Body type "${typeof body}" is not implemented`);
		}

		this.#body ??= new ReadableStream();
	}

	async arrayBuffer(): Promise<ArrayBuffer> {
		if (this.#used) {
			throw new Error("body aleady used");
		}

		this.#used = true;

		const output = [];
		for await (const next of this.body) {
			output.push(...new Uint8Array(next));
		}
		return new Uint8Array(output).buffer;
	}

	async text() {
		return DECODER.decode(await this.arrayBuffer());
	}

	async json() {
		return JSON.parse(await this.text());
	}
}
