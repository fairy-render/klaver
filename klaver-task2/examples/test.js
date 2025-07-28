import { triggerAsyncId, executionAsyncId,createHook } from 'node:async_hooks'


createHook({
    init(aid, ty, tid) {
       print(`${aid}, ${ty}, ${tid}`, executionAsyncId())
    },
    before(aid) {
        print("Before", aid)
    },
    after(aid) {
        print("After", aid)
    },
    destroy(aid) {
        print("Destroy", aid)
    }
})

// print("Test", executionAsyncId(), triggerAsyncId())

function timeout(ms) {
    return new Promise((res) => setTimeout(res, ms))
}

testAsync(() => {
    print('Hello, World! ' + executionAsyncId() + ' ' + triggerAsyncId())
    testAsync(() => {
        print('Hello, World! ' + executionAsyncId() + ' ' + triggerAsyncId())
        // throw new Error('dsds')
    })

    testAsync(() => {
        print('Hello, World! ' + executionAsyncId() + ' ' + triggerAsyncId())
        // throw new Error('dsds')
    })
})

setTimeout(() => {
    print("Timeout", executionAsyncId(), triggerAsyncId())
    setTimeout(() => {
        print("Timeout2", executionAsyncId(), triggerAsyncId())
    },500)
}, 200)

await timeout(1000)