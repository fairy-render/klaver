
export { }

// function sleep(ms: number = 0) {
//     return new Promise((res) => setTimeout(res, ms))
// }

// setTimeout(() => {
//     console.log('HEj');
    
// }, 100);

// await sleep(500)

console.log(await fetch("https://www.google.com/").then(m => m.text()))

// console.log('raprap')