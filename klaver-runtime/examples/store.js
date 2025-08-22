import { createHook, executionAsyncId, triggerAsyncId, AsyncLocalStorage } from 'node:async_hooks';

createHook({
    init: () => { }
}).enable()

if (typeof print !== 'function') {
    globalThis.print = console.log.bind(console);
}

print("Hello", executionAsyncId())

function delay(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const storage = new AsyncLocalStorage();
const snapshot = storage.run(new Map(), () => {
    print('Initial executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());

    setTimeout(() => {
        print('Inside first timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
        storage.getStore().set('key1', 'value1');
        // print("Stored value", storage.getStore().set('key1', 'value1'))
        setTimeout(() => {
            print('Inside second timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
            print('Stored value:', storage.getStore().get('key1'));
        }, 100);
    }, 100);

    setTimeout(() => {
        print("Another timeout")
    }, 100)

    return AsyncLocalStorage.snapshot();
});


const future = snapshot(() => {

    print('Snapshot', storage.getStore(), executionAsyncId())

    setTimeout(() => {
        print("Snapshot timeout", executionAsyncId(), storage.getStore())
    }, 100)

    return delay(500).then(() => {
        print('Snap After delay executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
        print('Snap Stored value after delay:', storage.getStore());
    })

})
print('store', storage.getStore())
