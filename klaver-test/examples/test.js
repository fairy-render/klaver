import { TestRunner, assert } from "@klaver/test";

const runner = new TestRunner();

runner.describe("Describe 1", () => {
  runner.it("should pass", () => {
    assert.ok(1 + 1 === 2);
  });

  runner.it("should pass async", async () => {
    await Promise.resolve();
    assert.equal(1, 1);
  });

  runner.it("should fail", () => {
    assert.deepEqual({ a: 1 }, { a: 2 });
  });

  runner.describe("Describe 2", () => {
    runner.it("should inner", () => {
      assert.deepEqual({ a: 1 }, { a: 1 });
    });
  });
});

await runner.run();
