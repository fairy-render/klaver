

declare var Worker: {
    new(scriptURL: string): WorkerInstance;
    prototype: WorkerInstance;
}

declare interface WorkerInstance {
    postMessage(message: any): void;
    onmessage: ((event: any) => void) | null;
    addEventListener(type: "message", listener: (event: any) => void): void;
    removeEventListener(type: "message", listener: (event: any) => void): void;
    terminate(): void;
}