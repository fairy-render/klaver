export class TestRunner {
  constructor();

  describe(description: string, fn: () => void): this;
  it(description: string, fn: () => void | Promise<void>): this;
  run(): Promise<void>;
}

export const assert: {
  ok(expr: unknown, message?: string): void;
  equal(actual: unknown, expected: unknown, message?: string): void;
  deepEqual(actual: unknown, expected: unknown, message?: string): void;
};
