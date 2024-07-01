function writeProp(out, name, value) {
    Object.defineProperty(out, name, {
        writable: !0,
        configurable: !0,
        enumerable: !0,
        value
    });
}
function writeProps(out, value) {
    for(let key in value)writeProp(out, key, value[key]);
}

class Console {
    #target;
    constructor(target = ""){
        this.#target = target;
    }
    log(...args) {
        log(...args);
    }
    warn(...args) {
        this.#print("warn", args);
    }
    error(...args) {
        this.#print("warn", args);
    }
    #print(level, args) {
        let formatted = args.map((m)=>Core.format(m)).join(" ");
        print(`[${level}] ${formatted}`);
    }
}
function log(...args) {
    let formatted = args.map((m)=>Core.format(m)).join(" ");
    print(`${formatted}`);
}
function init(global) {
    Object.defineProperties(global, {
        Console: {
            value: Console
        },
        console: {
            value: new Console()
        }
    });
}

async function main(global) {
    writeProps(global, {
        setTimeout: (cb, timeout)=>Core.timers.createTimer(cb, timeout, !1),
        clearTimeout: Core.timers.clearTimer.bind(Core.timers),
        setInterval: (cb, timeout)=>Core.timers.createTimer(cb, timeout, !0),
        clearInterval: Core.timers.clearTimer.bind(Core.timers)
    });
    let { TextEncoder, TextDecoder } = await import('@klaver/encoding');
    writeProps(global, {
        TextDecoder,
        TextEncoder
    }), init(global);
}

export { main as default };
