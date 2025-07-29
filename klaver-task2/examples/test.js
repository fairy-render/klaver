import { triggerAsyncId, executionAsyncId, createHook } from 'node:async_hooks'


print({
    triggerAsyncId: triggerAsyncId(),
    executionAsyncId: executionAsyncId()
})

createHook({
    init(aid, ty, tid) {
        print('Init', {
            id: aid,
            type: ty,
            triggerId: tid
        })
    },
    // before(aid) {
    //     print("Before", aid)
    // },
    // after(aid) {
    //     print("After", aid)
    // },
    destroy(aid) {
        print("Destroy", aid)
    }
})

// print("Test", executionAsyncId(), triggerAsyncId())


function timeout(ms) {
    return new Promise((res) => setTimeout(res, ms))
}

// testAsync(() => {
//     // print('Root ' + executionAsyncId() + ' ' + triggerAsyncId())
//     testAsync(() => {
//         print('Child1 ' + executionAsyncId() + ' ' + triggerAsyncId())
//         // throw new Error('dsds')
//     })

//     testAsync(() => {
//         print('Child2 ' + executionAsyncId() + ' ' + triggerAsyncId())
//         // throw new Error('dsds')
//     })
// })

async function test() {
    // print("Test", executionAsyncId(), triggerAsyncId())
    // await timeout(1000)
    // print("After timeout", executionAsyncId(), triggerAsyncId())
}

// setTimeout(() => {
//     print("Timeout", executionAsyncId(), triggerAsyncId())
//     setTimeout(() => {
//         print("Timeout2", executionAsyncId(), triggerAsyncId())
//     }, 500)
// }, 200)

// await test()

// const registry = new FinalizationRegistry((value) => {
//     print('FinalizationRegistry called for', value);
// })

// function test() {
//     print("Enter", executionAsyncId(), triggerAsyncId())
//     const future = new Promise((resolve, reject) => {
//         print("Test", executionAsyncId(), triggerAsyncId())
//         // setTimeout(() => {
//         //     print("Timeout", executionAsyncId(), triggerAsyncId())
//         //     resolve()
//         // }, 1000)
//     })

//     registry.register(future, 'Future Object');

//     return future;
// }



// async function rapper() {
//     await test()

// }


// test();


await timeout(3000)