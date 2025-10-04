import { createHook, executionAsyncId, triggerAsyncId, AsyncLocalStorage } from 'node:async_hooks';

console.log(executionAsyncId(), triggerAsyncId())


createHook({
    init(aid, ty, tid) {
        console.log(aid, ty, tid)
    }
}).enable()


setTimeout(() => {
    console.log('hello')
    console.log(executionAsyncId(), triggerAsyncId())
    setTimeout(() => {
        console.log(executionAsyncId(), triggerAsyncId())
    }, 0)

    setTimeout(() => {
        console.log(executionAsyncId(), triggerAsyncId())
        setTimeout(() => {
            console.log(executionAsyncId(), triggerAsyncId())
        }, 0)
    }, 0)
}, 0)


