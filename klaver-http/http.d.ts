export type Handler = (req: Request) => Promise<Response> | Response;

export class Router {
	get(path: string, handler: Handler): void;
	post(path: string, handler: Handler): void;
	patch(path: string, handler: Handler): void;
	put(path: string, handler: Handler): void;
	delete(path: string, handler: Handler): void;
	head(path: string, handler: Handler): void;
	any(path: string, handler: Handler): void;
}

export interface ServeOptions {
	port?: number;
}

export function serve(router: Router, opts: ServeOptions): Promise<void>;
