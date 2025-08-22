import { executionAsyncId } from 'node:async_hooks'


function sleep(ms) {
    print("Create sleep promise", executionAsyncId())
    return new Promise(resolve => {
        print("Sleeping for", ms, "ms", executionAsyncId())
        setTimeout(resolve, ms)
    }).then(ret => {
        print("1 After sleep promise", executionAsyncId())
    });
}

await sleep(200).then(ret => {
    print("After sleep promise", executionAsyncId())
});

// const promise = new Promise((resolve, reject) => {
//     print('Creating promise', executionAsyncId());
//     resolve('Promise resolved');
// }).then(result => {
//     print(result, 'in then block', executionAsyncId());
// });

// await promise;