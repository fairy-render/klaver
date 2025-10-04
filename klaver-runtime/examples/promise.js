import { executionAsyncId, triggerAsyncId, createHook } from 'node:async_hooks'


createHook({
    init: () => { }
}).enable()

if (typeof print !== 'function') {
    globalThis.print = console.log.bind(console);
}

print("Root", executionAsyncId(), triggerAsyncId())

function sleep(ms) {
    print("Create sleep promise", executionAsyncId(), triggerAsyncId())
    return new Promise(resolve => {
        print("Sleeping for", ms, "ms", executionAsyncId(), triggerAsyncId())
        setTimeout(resolve, ms)
    }).then(ret => {
        print("1 After sleep promise", executionAsyncId(), triggerAsyncId())
    });
}

sleep(200).then(ret => {
    print("After sleep promise", executionAsyncId(), triggerAsyncId())
}).then(() => [
    print("After after")
]);

print("Root", executionAsyncId(), triggerAsyncId())


// const promise = new Promise((resolve, reject) => {
//     print('Creating promise', executionAsyncId());
//     resolve('Promise resolved');
// }).then(result => {
//     print(result, 'in then block', executionAsyncId());
// });

// await promise;