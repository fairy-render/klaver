declare function print(...args: unknown[]): void;

declare type TimeId = number;

declare interface Timers {
	createTimer(cb: () => void, delay: number, repeat?: boolean): TimeId;
	clearTimer(id: TimeId): void;
}

declare interface FormatOptions {
	colors: boolean;
}

declare interface CoreApi {
	readonly timers: Timers;
	readonly format: (value: unknown, options?: FormatOptions) => string;
}

declare const Core: CoreApi;
