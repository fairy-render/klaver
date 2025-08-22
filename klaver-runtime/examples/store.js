import { createHook, executionAsyncId, triggerAsyncId, AsyncLocalStorage } from 'node:async_hooks';



const storage = new AsyncLocalStorage();
storage.run(new Map(), () => {
    print('Initial executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());

    setTimeout(() => {
        print('Inside first timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
        // storage.getStore().set('key1', 'value1');
        print("Stored value", storage.getStore().set('key1', 'value1'))
        setTimeout(() => {
            print('Inside second timeout executionAsyncId:', executionAsyncId(), 'triggerAsyncId:', triggerAsyncId());
            print('Stored value:', storage.getStore().get('key1'));
        }, 100);
    }, 100);

    setTimeout(() => {
        print("Another timeout")
    }, 100)
});


