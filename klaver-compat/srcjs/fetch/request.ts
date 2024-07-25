import {
	Headers,
	type Method,
	Request as KlaverRequest,
	Url,
} from "@klaver/http";
import type { AbortSignal } from "abort-controller";
import { Body } from "./body.js";
import { ReadableStream } from "web-streams-polyfill";

export class Request extends Body {
	#url: Url;
	#headers: Headers;
	#method: Method;
	#signal?: AbortSignal;

	get method(): Method {
		return this.#method;
	}

	get url(): Url {
		return this.#url;
	}

	get headers(): Headers {
		return this.#headers;
	}

	constructor(input: Request | string | Url, init: RequestInit = {}) {
		super(
			init.body ??
				(input && input instanceof Request
					? input.body
					: input && input instanceof KlaverRequest
						? input.stream()
						: void 0),
		);

		if (input && input instanceof Request) {
			this.#url = input.#url;
			this.#headers = input.#headers;
			this.#method = input.#method;
			this.#signal = this.#signal;
		} else if (input && input instanceof Url) {
			this.#url = input;
		} else if (typeof input === "string") {
			this.#url = new Url(input);
		}

		this.#headers ??= new Headers();
		this.#method ??= "GET";
	}

	clone() {
		return new Request(this);
	}
}
