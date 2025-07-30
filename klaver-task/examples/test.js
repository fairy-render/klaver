import { triggerAsyncId, executionAsyncId, createHook, executionAsyncResource, resourceName } from 'node:async_hooks'



createHook({
    init(aid, ty, tid, resource) {
        print("Name", resourceName(ty))
        const cr = executionAsyncResource();
        if (cr) {
            resource["test"] = cr["test"];
        }
        // print('Init', {
        //     id: aid,
        //     type: ty,
        //     triggerId: tid,
        //     resource: resource,
        //     executionAsyncId: executionAsyncId(),
        //     triggerAsyncId: triggerAsyncId(),

        // })
    },
}).enable()



print("Eis", executionAsyncId())

setTimeout(() => {
    executionAsyncResource()['test'] = 42;
    print("Eis", executionAsyncId())
    setTimeout(() => {
        print("Eis", executionAsyncId())
        print(executionAsyncResource())
        setTimeout(() => {
            print("Eis", executionAsyncId())
            print(executionAsyncResource())
        })
    })

    new Promise((res) => {
        print("Promise", executionAsyncId(), triggerAsyncId());
        print(executionAsyncResource())
        res()
    })

})


// setTimeout(() => {
//     executionAsyncResource()['test'] = 43;
//     print("Eis2", executionAsyncId())
//     setTimeout(() => {
//         print("Eis2", executionAsyncId())
//         print(executionAsyncResource())
//         setTimeout(() => {
//             print("Eis2", executionAsyncId())
//             print(executionAsyncResource())
//         })
//     })
// })
// setTimeout(() => {
//     print('hello')
//     print(executionAsyncId(), triggerAsyncId())
//     setTimeout(() => {
//         print(executionAsyncId(), triggerAsyncId())
//     }, 0)

//     setTimeout(() => {
//         print(executionAsyncId(), triggerAsyncId())
//         setTimeout(() => {
//             print(executionAsyncId(), triggerAsyncId())
//         }, 0)
//     }, 0)
// }, 0)



// print({
//     triggerAsyncId: triggerAsyncId(),
//     executionAsyncId: executionAsyncId(),
//     hooks: globalThis.$__hooks
// })


// createHook({
//     init(aid, ty, tid, resource) {
//         resource["seen"] = executionAsyncResource();

// print('Init', {
//     id: aid,
//     type: ty,
//     triggerId: tid,
//     resource: resource,
//     executionAsyncId: executionAsyncId(),
//     triggerAsyncId: triggerAsyncId(),

// })
//     },
//     // before(aid) {
//     //     print("Before", aid)
//     // },
//     // after(aid) {
//     //     print("After", aid)
//     // },
//     destroy(aid) {
//         print("Destroy", aid)
//     },
//     promiseResolve: (id) => {
//         print("Promise resolve", id, triggerAsyncId(), executionAsyncId())
//     }
// })

// // print("Test", executionAsyncId(), triggerAsyncId())


// // function timeout(ms) {
// //     console.log(executionAsyncId())

// //     return new Promise((res) => setTimeout(res, ms))
// // }

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

// async function test() {
//     // print("Test", executionAsyncId(), triggerAsyncId())
//     // await timeout(1000)
//     // print("After timeout", executionAsyncId(), triggerAsyncId())
// }

// // setTimeout(() => {
// //     print("Timeout", executionAsyncId(), triggerAsyncId())
// //     setTimeout(() => {
// //         print("Timeout2", executionAsyncId(), triggerAsyncId())
// //     }, 500)
// // }, 200)

// // await test()

// // const registry = new FinalizationRegistry((value) => {
// //     print('FinalizationRegistry called for', value);
// // })

// // function test() {
// //     print("Enter", executionAsyncId(), triggerAsyncId())
// //     const future = new Promise((resolve, reject) => {
// //         print("Test", executionAsyncId(), triggerAsyncId())
// //         // setTimeout(() => {
// //         //     print("Timeout", executionAsyncId(), triggerAsyncId())
// //         //     resolve()
// //         // }, 1000)
// //     })

// //     registry.register(future, 'Future Object');

// //     return future;
// // }



// // async function rapper() {
// //     await test()

// // }


// // test();


// await timeout(1000)


