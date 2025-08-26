import {AsyncResource, executionAsyncId} from 'node:async_hooks'


if (typeof print !== 'function') {
  globalThis.print = console.log.bind(console);
}

const resource = new AsyncResource("");



print("Root", executionAsyncId(), "Resource", resource.asyncId());

function delay(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function test() {
  print("TEST", executionAsyncId())
  await test2()
}

async function test2() {
  print("TEST 2", executionAsyncId())
  await delay(500)
}

await resource.runInAsyncScope(() => {
  print("HEllo", executionAsyncId())
  return test()
})

print("Root", executionAsyncId(), "Resource", resource.asyncId());
