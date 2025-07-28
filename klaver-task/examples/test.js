

print(currentAsyncId() + ' ' + currentExecutionId())

createHook({
    init: (id, ty, target) => {
        print(`Init ${id}, ${ty}, ${target}: ${currentAsyncId()} ${currentExecutionId()}`)
    },
    before(id) {
        print(`Before ${id}: ${currentAsyncId()} ${currentExecutionId()}`)
    }
})

testAsync(() => {
    print('Hello, World! ' + currentAsyncId() + ' ' + currentExecutionId())
    testAsync(() => {
        print('Hello, World!' + currentAsyncId() + ' ' + currentExecutionId())
    })

    testAsync(() => {
        print('Hello, World!' + currentAsyncId() + ' ' + currentExecutionId())
    })
})