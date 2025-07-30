
export { }

console.log('HEj');
const ret = await fetch('https://google.com').then(async res => {
    try {
        return await res.text();
    } catch (e) {
        console.log('Got error: ', e)
        return ""
    }

});
console.log('return', ret);