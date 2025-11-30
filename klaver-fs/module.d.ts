

export interface File {
  read(len: number): Promise<ArrayBuffer>;
  arrayBuffer(): Promise<ArrayBuffer>;
  write(buffer: ArrayBuffer): Promise<void>;
}


export interface FileSystem {
  readonly name: string;
  readonly root: FileSystemEntry
}

export interface FileSystemEntry {
  readonly fileName:string;
  readonly extension: string;
  
  toString(): string;
  resolve(path: string): FileSystemEntry;
  metadata(): Promise<Metadata>;
  listDir(): Promise<IterableIterator<FileSystemEntry>>;
  open(opts: OpenOptions): Promise<File>
}

export interface Metadata {
  size: number;
  type: 'dir' | 'file'
}

export interface OpenOptions {
  read?: boolean;
  write?: boolean;
  append?: boolean;
  create?: boolean;
  truncate?: boolean;
}


export function open(path: string): Promise<FileSystem>;