import { createHook, executionAsyncId, triggerAsyncId, AsyncLocalStorage } from 'node:async_hooks';


const snapshot = AsyncLocalStorage.snapshot();

const storage = new AsyncLocalStorage();
storage.run(new Map(), () => {
    console.log('Initial executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());

    setTimeout(() => {
        console.log('Inside first timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
        storage.getStore().set('key1', 'value1');

        setTimeout(() => {
            console.log('Inside second timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
            console.log('Stored value:', storage.getStore().get('key1'));
        }, 100);
    }, 100);
});


const snapshot2 = storage.run(42, () => AsyncLocalStorage.snapshot());

snapshot2(() => {
    console.log("Inside snapshot2 executionAsyncId:", executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
    console.log('Stored value in snapshot2:', storage.getStore());
})


console.log(typeof storage.getStore())

setTimeout(() => {
    console.log('timout');
    console.log('Initial executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());

    console.log(typeof storage.getStore()); // Should be undefined, as this is outside the AsyncLocalStorage context
}, 300);

