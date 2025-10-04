import { Timers } from 'timer'

const timer = new Timers();

timer.createTimeout(() => {
    print("!")
}, 600, false)


timer.createTimeout(() => {
    print("World")
}, 200, false)

const id = timer.createTimeout(() => {
    print("Butt")
}, 700, false)

timer.createTimeout(() => {
    timer.clearTimeout(id)
}, 400)

print("Hello")

// setTimeout(() => {
//     throw new Error("Ger")
//     print("Timeout!")
// }, 2000)