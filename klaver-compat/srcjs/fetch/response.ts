import { Body } from "./body.js";
import { Headers, Response as KlaverResponse } from "@klaver/http";
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

		if (init?.headers) {
			if (init.headers instanceof Headers) {
				this.#headers = init.headers;
			} else if (Array.isArray(init.headers)) {
				const headers = new Headers();
				for (const [k, v] of init.headers) {
					headers.append(k, v);
				}
				this.#headers = headers;
			} else {
				const headers = new Headers();
				for (const k in init.headers) {
					headers.append(k, init.headers[k]);
				}

				this.#headers = headers;
			}
		}
	}
}
