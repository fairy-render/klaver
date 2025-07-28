// import { createHook, executionAsyncId, triggerAsyncId } from 'node:async_hooks';

// console.log(executionAsyncId(), triggerAsyncId())

// createHook({
//   init: (asyncId, type, triggerId, resource) => {
//     console.log(`asyncID ${asyncId}, type: ${type}, triggerAsyncId ${triggerId}, exectionId ${executionAsyncId()} triggerAsyncId ${triggerAsyncId()}`)
//   }
// }).enable()



// setTimeout(() => {
//   console.log('hello')
// }, 200)


setTimeout(() => {
    console.log("Timeout")
    setTimeout(() => {
        console.log("Timeout2")
    },200)
}, 200)