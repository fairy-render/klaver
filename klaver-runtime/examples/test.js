import { triggerAsyncId, executionAsyncId, executionAsyncResource, resourceName, createHook } from 'node:async_hooks'


// const print = console.log.bind(console);


createHook({
    init(aid, ty, tid, resource) {
        // print("Name", aid, resourceName(ty))
        print('init', aid, tid, resourceName(ty))
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
  print('hello')
  executionAsyncResource()["test"] = 42;
  print(executionAsyncId(), triggerAsyncId())
  setTimeout(() => {
      print(executionAsyncId(), triggerAsyncId())
  }, 0)

  setTimeout(() => {
      print(executionAsyncId(), triggerAsyncId())
      setTimeout(() => {
          print(executionAsyncId(), triggerAsyncId())
      }, 0)

      print(executionAsyncResource())
  }, 0)


}, 0)


