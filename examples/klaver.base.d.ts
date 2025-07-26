/// <reference no-default-lib="true"/>

/// <reference lib="es2020" />


// EventTarget

interface EventListener {
    (evt: Event): void;
}

interface EventListenerObject {
    handleEvent(object: Event): void;
}

type EventListenerOrEventListenerObject = EventListener | EventListenerObject;

interface EventSourceEventMap {
    "error": Event;
}

interface Event { }

declare var Event: {
    prototype: Event,
    new(type: String): Event
};

interface EventTarget {
    addEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
    removeEventListener(type: string, listener: EventListenerOrEventListenerObject): void;
    dispatchEvent(event: Event): boolean;
}

declare var EventTarget: {
    prototype: EventTarget;
    new(): EventTarget;
}


// Console

interface Console {
    log(...data: any[]): void;
    warn(...data: any[]): void;
    error(...data: any[]): void;
    info(...data: any[]): void;
    debug(...data: any[]): void;
    time(label?: string): void;
    timeEnd(label?: string): void;
}

declare var console: Console;


// AbortController

interface AbortSignal {
    readonly aborted: boolean;
    onabort: ((this: AbortSignal, ev: Event) => any) | null;
    addEventListener(type: "abort", listener: (this: AbortSignal, ev: Event) => any): void;
    removeEventListener(type: "abort", listener: (this: AbortSignal, ev: Event) => any): void;
}

declare var AbortSignal: {
    prototype: AbortSignal;
    new(): AbortSignal;
};

interface AbortController {
    readonly signal: AbortSignal;
    abort(): void;
    onabort: ((this: AbortController, ev: Event) => any) | null;
}

declare var AbortController: {
    prototype: AbortController;
    new(): AbortController;
};