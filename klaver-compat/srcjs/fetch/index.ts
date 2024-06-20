import { AbortController, AbortSignal } from "abort-controller";
import type {
	RequestInit as KlaverRequestInit,
	Response as KlaverResponse,
} from "@klaver/http";
import { URL, URLParts, URLSearchParams } from "./url.js";

const http = await import("@klaver/http");

const CLIENT = new http.Client();

export default async function init(global: Record<string, unknown>) {
	Object.defineProperty(global, "fetch", {
		value: fetchImpl,
		configurable: true,
		writable: true,
	});

	Object.defineProperties(global, {
		AbortController: {
			value: AbortController,
		},
		AbortSignal: {
			value: AbortSignal,
		},
		URL: {
			value: URL,
		},
		URLSearchParams: {
			value: URLSearchParams,
		},
		Request: {
			value: class Request {},
		},
		Response: {
			value: class Response {},
		},
	});
}

function fetchImpl(input: RequestInfo | URL, init?: RequestInit | undefined) {
	const opts: KlaverRequestInit = {
		// method: init?.method ?? "GET",
		headers: new http.Headers(),
	};

	const url = typeof input === "string" ? input : input;

	if (init?.headers) {
		let headers = init.headers;
		if (!Array.isArray(init.headers)) {
			headers = Object.entries(init.headers);
		}
	}

	if (init?.signal) {
		opts.cancel = new http.Cancel();
		init.signal.onabort = opts.cancel.cancel.bind(opts.cancel);
	}

	const req = new http.Request(url?.toString(), opts);

	return CLIENT.send(req);
}

export class Response {
	#body: ReadableStream<ArrayBuffer>;
	#inner?: KlaverResponse;
	constructor(body?: ReadableStream | KlaverResponse) {
		if (body && body instanceof http.Response) {
			this.#inner = body;
			let stream: AsyncIterator<ArrayBuffer>;
			this.#body = new ReadableStream({
				async pull(controller) {
					const { done, value } = await stream.next();
					if (done) {
						controller.close();
					} else {
						controller.enqueue(value);
					}
				},
				async start(controller) {
					stream = (await body.stream())[Symbol.asyncIterator]();
				},
			});
		}
	}
}
