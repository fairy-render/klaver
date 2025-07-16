import { Timers } from 'timer'

const timer = new Timers();

timer.createTimeout(() => {
    print("!")
}, 100, false)


const id = timer.createTimeout(() => {
    print("World")
}, 800, false)

timer.createTimeout(() => {
    timer.clearTimeout(id)
}, 400)

print("Hello")

setTimeout(() => {
    throw new Error("Ger")
    print("Timeout!")
}, 2000)