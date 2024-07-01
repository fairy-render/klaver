function writeProp(out, name, value) {
    Object.defineProperty(out, name, {
        writable: !0,
        configurable: !0,
        enumerable: !0,
        value
    });
}
let Core = globalThis.Core;
async function main(global) {
    writeProp(global, "setTimeout", (cb, timeout)=>Core.timers.createTimer(cb, timeout, !1)), writeProp(global, "clearTimeout", Core.timers.clearTimer.bind(Core.timers)), writeProp(global, "setInterval", (cb, timeout)=>Core.timers.createTimer(cb, timeout, !0)), writeProp(global, "clearInterval", Core.timers.clearTimer.bind(Core.timers));
    let { TextEncoder, TextDecoder } = await import('@klaver/encoding');
    writeProp(global, "TextEncoder", TextEncoder), writeProp(global, "TextDecoder", TextDecoder);
}

export { main as default };
