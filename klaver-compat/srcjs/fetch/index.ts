import AbortController, { AbortSignal } from "abort-controller";
import { lazy, writeProps } from "../util.js";
import { Request } from "./request.js";
import { Response } from "./response.js";
import { URL, URLSearchParams } from "./url.js";
import { Cancel, Client } from "@klaver/http";

export default async function init(global: Record<string, unknown>) {
	const {
		Request: KlaverRequest,
		Response: KlaverResponse,
		Headers,
		Client,
		createCancel,
	} = await import("@klaver/http");

	const client = lazy(() => new Client());

	writeProps(global, {
		URL,
		Response,
		Request,
		Headers,
		URLSearchParams,
		AbortController,
		AbortSignal,
		fetch(url: string | Request | URL, init?: RequestInit): Promise<Response> {
			return fetchImpl(url, init);
		},
	});

	async function fetchImpl(
		url?: string | Request | URL,
		init?: RequestInit,
	): Promise<Response> {
		const req = new Request(url, init);

		let cancel: Cancel | undefined;
		if (init?.signal) {
			const signal = init.signal;
			cancel = new Cancel();
			signal.addEventListener("abort", () => {
				cancel.cancel();
			});
		}

		const httpReq = new KlaverRequest(req.url.toString(), {
			method: req.method,
			headers: req.headers,
			cancel,
		});

		const resp = await client().send(httpReq);

		return new Response(resp);
	}
}
