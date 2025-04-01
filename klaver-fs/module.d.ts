export type Buffer = ArrayBuffer | Uint8Array;

export interface DirEntry {
  path: string;
  type: "file" | "dir" | "symlink";
}

export type OpenFlag = "r" | "w" | "a" | "t";

export interface File {
  readLines(): Promise<AsyncIterableIterator<string>>;
  write(buffer: Buffer): Promise<void>;
  read(buffer: Buffer): Promise<number>;
  flush(): Promise<void>;
  close(): Promise<void>;
}

export function read(path: string): Promise<ArrayBuffer>;

export function write(path: string, content: Buffer): Promise<void>;

export function readDir(path: string): Promise<AsyncIterable<DirEntry>>;

export function resolve(path: string): Promise<string>;

export function open(path: string, flag?: OpenFlag): Promise<File>;

export function mkdir(path: string): Promise<void>;

export function exists(path: string): Promise<boolean>;
