export type Next = { call: (req: Request) => Promise<Response> };

export type Handler = (req: Request) => Promise<Response> | Response;

export type Middleware = (req: Request, next: Next) => Promise<Response>;

export type Method =
	| "GET"
	| "POST"
	| "PUT"
	| "PATCH"
	| "DELETE"
	| "HEAD"
	| "OPTIONS";

export class Router {
	constructor();
	get(path: string, handler: Handler): void;
	post(path: string, handler: Handler): void;
	patch(path: string, handler: Handler): void;
	put(path: string, handler: Handler): void;
	delete(path: string, handler: Handler): void;
	head(path: string, handler: Handler): void;
	any(path: string, handler: Handler): void;
	route(method: Method, path: string, handler: Handler): void;
	use(middleware: Middleware): void;
}

export interface ServeOptions {
	port?: number;
	debug?: boolean;
}

export function serve(router: Router, opts: ServeOptions): Promise<void>;
