import { Headers, type Method } from "@klaver/http";
import { URL } from "./url.js";
import type { AbortSignal } from "abort-controller";
import { Body } from "./body.js";
import { ReadableStream } from "web-streams-polyfill";

export class Request extends Body {
	#url: URL;
	#headers: Headers;
	#method: Method;
	#signal?: AbortSignal;

	get method(): Method {
		return this.#method;
	}

	get url(): URL {
		return this.#url;
	}

	get headers(): Headers {
		return this.#headers;
	}

	constructor(input: Request | string | URL, init: RequestInit = {}) {
		super(
			init.body ??
				(input && input instanceof Request ? input.body : new ReadableStream()),
		);

		if (input && input instanceof Request) {
			this.#url = input.#url;
			this.#headers = input.#headers;
			this.#method = input.#method;
			this.#signal = this.#signal;
		} else if (input && input instanceof URL) {
			this.#url = input;
		} else if (typeof input === "string") {
			this.#url = new URL(input);
		}

		this.#headers ??= new Headers();
		this.#method ??= "GET";
	}

	clone() {
		return new Request(this);
	}
}