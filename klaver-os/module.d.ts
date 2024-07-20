export class Exec {
	output(): Promise<ArrayBuffer>;
	pipe(exec: Exec): Pipe;
}

export class Pipe {
	output(): Promise<ArrayBuffer>;
	pipe(exec: Exec): Pipe;
}

export function cat(path: string): Promise<AsyncIterable<ArrayBuffer>>;
export function sh(cmd: string, ...rest: string[]): Exec;
