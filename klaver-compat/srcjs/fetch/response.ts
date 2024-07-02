import { Body } from "./body.js";
import { Response as KlaverResponse, type Headers } from "@klaver/http";
export class Response extends Body {
	#status: number;
	#headers: Headers;

	get status() {
		return this.#status;
	}

	get headers() {
		return this.#headers;
	}

	constructor(body?: BodyInit | KlaverResponse, init: ResponseInit = {}) {
		super(
			body && body instanceof KlaverResponse
				? body.stream()
				: new ReadableStream(),
		);

		if (body && body instanceof KlaverResponse) {
			this.#status = body.status;
			this.#headers = body.headers;
		}
	}
}
