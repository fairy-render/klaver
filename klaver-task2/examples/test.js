import { triggerAsyncId, executionAsyncId } from 'node:async_hooks'

print("Test", executionAsyncId(), triggerAsyncId())

testAsync(() => {
    print('Hello, World! ' + executionAsyncId() + ' ' + triggerAsyncId())
    testAsync(() => {
        print('Hello, World! ' + executionAsyncId() + ' ' + triggerAsyncId())
    })
})