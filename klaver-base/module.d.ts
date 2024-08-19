declare class Event {
	readonly type: string;
	constructor(type: string);
}

declare class CustomEvent extends Event {
	readonly details: unknown;
	constructor(type: string, options?: { details?: unknown });
}

declare class EventTarget {
	constructor();

	addEventListener(event: string, listener: (event: Event) => void): void;
	removeEventListener(event: string, listener: (event: Event) => void): void;
	dispatchEvent(event: Event): void;
}

declare class AbortController {
	readonly signal: AbortSignal;

	abort(reason?: unknown): void;
}

declare class AbortSignal extends EventTarget {
	readonly aborted: boolean;
	readonly reason?: unknown;
}
