

export class Vm {
  static open(): Promise<Vm>;
  
  evalPath(path: string): Promise<unknown>;
  eval(source: string): Promise<unknown>;
}

