

interface File {
  read(len: number): Promise<ArrayBuffer>;
  arrayBuffer(): Promise<ArrayBuffer>;
  write(buffer: ArrayBuffer): Promise<void>;
}


interface FileSystem {
  readonly name: string;
  readonly root: FileSystemEntry
}

interface FileSystemEntry {
  readonly fileName:string;
  readonly extension: string;
  
  toString(): string;
  resolve(path: string): FileSystemEntry;
  metadata(): Promise<Metadata>;
  listDir(): Promise<IterableIterator<FileSystemEntry>>;
  open(opts: OpenOptions): Promise<File>
}

interface Metadata {
  size: number;
  type: 'dir' | 'file'
}

interface OpenOptions {
  read?: boolean;
  write?: boolean;
  append?: boolean;
  create?: boolean;
  truncate?: boolean;
}