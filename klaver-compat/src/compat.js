let privateData = new WeakMap(), wrappers = new WeakMap();
function pd(event) {
    let retv = privateData.get(event);
    return console.assert(null != retv, "'this' is expected an Event object, but got", event), retv;
}
function setCancelFlag(data) {
    if (null != data.passiveListener) {
        "undefined" != typeof console && "function" == typeof console.error && console.error("Unable to preventDefault inside passive event listener invocation.", data.passiveListener);
        return;
    }
    data.event.cancelable && (data.canceled = !0, "function" == typeof data.event.preventDefault && data.event.preventDefault());
}
function Event(eventTarget, event) {
    privateData.set(this, {
        eventTarget,
        event,
        eventPhase: 2,
        currentTarget: eventTarget,
        canceled: !1,
        stopped: !1,
        immediateStopped: !1,
        passiveListener: null,
        timeStamp: event.timeStamp || Date.now()
    }), Object.defineProperty(this, "isTrusted", {
        value: !1,
        enumerable: !0
    });
    let keys = Object.keys(event);
    for(let i = 0; i < keys.length; ++i){
        let key = keys[i];
        key in this || Object.defineProperty(this, key, defineRedirectDescriptor(key));
    }
}
function defineRedirectDescriptor(key) {
    return {
        get () {
            return pd(this).event[key];
        },
        set (value1) {
            pd(this).event[key] = value1;
        },
        configurable: !0,
        enumerable: !0
    };
}
function setPassiveListener(event, passiveListener) {
    pd(event).passiveListener = passiveListener;
}
Event.prototype = {
    get type () {
        return pd(this).event.type;
    },
    get target () {
        return pd(this).eventTarget;
    },
    get currentTarget () {
        return pd(this).currentTarget;
    },
    composedPath () {
        let currentTarget = pd(this).currentTarget;
        return null == currentTarget ? [] : [
            currentTarget
        ];
    },
    get NONE () {
        return 0;
    },
    get CAPTURING_PHASE () {
        return 1;
    },
    get AT_TARGET () {
        return 2;
    },
    get BUBBLING_PHASE () {
        return 3;
    },
    get eventPhase () {
        return pd(this).eventPhase;
    },
    stopPropagation () {
        let data = pd(this);
        data.stopped = !0, "function" == typeof data.event.stopPropagation && data.event.stopPropagation();
    },
    stopImmediatePropagation () {
        let data = pd(this);
        data.stopped = !0, data.immediateStopped = !0, "function" == typeof data.event.stopImmediatePropagation && data.event.stopImmediatePropagation();
    },
    get bubbles () {
        return !!pd(this).event.bubbles;
    },
    get cancelable () {
        return !!pd(this).event.cancelable;
    },
    preventDefault () {
        setCancelFlag(pd(this));
    },
    get defaultPrevented () {
        return pd(this).canceled;
    },
    get composed () {
        return !!pd(this).event.composed;
    },
    get timeStamp () {
        return pd(this).timeStamp;
    },
    get srcElement () {
        return pd(this).eventTarget;
    },
    get cancelBubble () {
        return pd(this).stopped;
    },
    set cancelBubble (value){
        if (!value) return;
        let data = pd(this);
        data.stopped = !0, "boolean" == typeof data.event.cancelBubble && (data.event.cancelBubble = !0);
    },
    get returnValue () {
        return !pd(this).canceled;
    },
    set returnValue (value){
        value || setCancelFlag(pd(this));
    },
    initEvent () {}
}, Object.defineProperty(Event.prototype, "constructor", {
    value: Event,
    configurable: !0,
    writable: !0
}), "undefined" != typeof window && void 0 !== window.Event && (Object.setPrototypeOf(Event.prototype, window.Event.prototype), wrappers.set(window.Event.prototype, Event));
let listenersMap = new WeakMap();
function isObject(x) {
    return null !== x && "object" == typeof x;
}
function getListeners(eventTarget) {
    let listeners = listenersMap.get(eventTarget);
    if (null == listeners) throw TypeError("'this' is expected an EventTarget object, but got another value.");
    return listeners;
}
function defineEventAttribute(eventTargetPrototype, eventName) {
    Object.defineProperty(eventTargetPrototype, `on${eventName}`, {
        get () {
            let node = getListeners(this).get(eventName);
            for(; null != node;){
                if (3 === node.listenerType) return node.listener;
                node = node.next;
            }
            return null;
        },
        set (listener) {
            "function" == typeof listener || isObject(listener) || (listener = null);
            let listeners = getListeners(this), prev = null, node = listeners.get(eventName);
            for(; null != node;)3 === node.listenerType ? null !== prev ? prev.next = node.next : null !== node.next ? listeners.set(eventName, node.next) : listeners.delete(eventName) : prev = node, node = node.next;
            if (null !== listener) {
                let newNode = {
                    listener,
                    listenerType: 3,
                    passive: !1,
                    once: !1,
                    next: null
                };
                null === prev ? listeners.set(eventName, newNode) : prev.next = newNode;
            }
        },
        configurable: !0,
        enumerable: !0
    });
}
function defineCustomEventTarget(eventNames) {
    function CustomEventTarget() {
        EventTarget.call(this);
    }
    CustomEventTarget.prototype = Object.create(EventTarget.prototype, {
        constructor: {
            value: CustomEventTarget,
            configurable: !0,
            writable: !0
        }
    });
    for(let i = 0; i < eventNames.length; ++i)defineEventAttribute(CustomEventTarget.prototype, eventNames[i]);
    return CustomEventTarget;
}
function EventTarget() {
    if (this instanceof EventTarget) {
        listenersMap.set(this, new Map());
        return;
    }
    if (1 == arguments.length && Array.isArray(arguments[0])) return defineCustomEventTarget(arguments[0]);
    if (arguments.length > 0) {
        let types = Array(arguments.length);
        for(let i = 0; i < arguments.length; ++i)types[i] = arguments[i];
        return defineCustomEventTarget(types);
    }
    throw TypeError("Cannot call a class as a function");
}
EventTarget.prototype = {
    addEventListener (eventName, listener, options) {
        if (null == listener) return;
        if ("function" != typeof listener && !isObject(listener)) throw TypeError("'listener' should be a function or an object.");
        let listeners = getListeners(this), optionsIsObj = isObject(options), listenerType = (optionsIsObj ? options.capture : options) ? 1 : 2, newNode = {
            listener,
            listenerType,
            passive: optionsIsObj && !!options.passive,
            once: optionsIsObj && !!options.once,
            next: null
        }, node = listeners.get(eventName);
        if (void 0 === node) {
            listeners.set(eventName, newNode);
            return;
        }
        let prev = null;
        for(; null != node;){
            if (node.listener === listener && node.listenerType === listenerType) return;
            prev = node, node = node.next;
        }
        prev.next = newNode;
    },
    removeEventListener (eventName, listener, options) {
        if (null == listener) return;
        let listeners = getListeners(this), listenerType = (isObject(options) ? options.capture : options) ? 1 : 2, prev = null, node = listeners.get(eventName);
        for(; null != node;){
            if (node.listener === listener && node.listenerType === listenerType) {
                null !== prev ? prev.next = node.next : null !== node.next ? listeners.set(eventName, node.next) : listeners.delete(eventName);
                return;
            }
            prev = node, node = node.next;
        }
    },
    dispatchEvent (event) {
        if (null == event || "string" != typeof event.type) throw TypeError('"event.type" should be a string.');
        let listeners = getListeners(this), eventName = event.type, node = listeners.get(eventName);
        if (null == node) return !0;
        let wrappedEvent = new (function getWrapper(proto) {
            if (null == proto || proto === Object.prototype) return Event;
            let wrapper = wrappers.get(proto);
            return null == wrapper && (wrapper = function(BaseEvent, proto) {
                let keys = Object.keys(proto);
                if (0 === keys.length) return BaseEvent;
                function CustomEvent(eventTarget, event) {
                    BaseEvent.call(this, eventTarget, event);
                }
                CustomEvent.prototype = Object.create(BaseEvent.prototype, {
                    constructor: {
                        value: CustomEvent,
                        configurable: !0,
                        writable: !0
                    }
                });
                for(let i = 0; i < keys.length; ++i){
                    let key = keys[i];
                    if (!(key in BaseEvent.prototype)) {
                        let isFunc = "function" == typeof Object.getOwnPropertyDescriptor(proto, key).value;
                        Object.defineProperty(CustomEvent.prototype, key, isFunc ? function(key) {
                            return {
                                value () {
                                    let event = pd(this).event;
                                    return event[key].apply(event, arguments);
                                },
                                configurable: !0,
                                enumerable: !0
                            };
                        }(key) : defineRedirectDescriptor(key));
                    }
                }
                return CustomEvent;
            }(getWrapper(Object.getPrototypeOf(proto)), proto), wrappers.set(proto, wrapper)), wrapper;
        }(Object.getPrototypeOf(event)))(this, event), prev = null;
        for(; null != node;){
            if (node.once ? null !== prev ? prev.next = node.next : null !== node.next ? listeners.set(eventName, node.next) : listeners.delete(eventName) : prev = node, setPassiveListener(wrappedEvent, node.passive ? node.listener : null), "function" == typeof node.listener) try {
                node.listener.call(this, wrappedEvent);
            } catch (err) {
                "undefined" != typeof console && "function" == typeof console.error && console.error(err);
            }
            else 3 !== node.listenerType && "function" == typeof node.listener.handleEvent && node.listener.handleEvent(wrappedEvent);
            if (pd(wrappedEvent).immediateStopped) break;
            node = node.next;
        }
        return setPassiveListener(wrappedEvent, null), pd(wrappedEvent).eventPhase = 0, pd(wrappedEvent).currentTarget = null, !wrappedEvent.defaultPrevented;
    }
}, Object.defineProperty(EventTarget.prototype, "constructor", {
    value: EventTarget,
    configurable: !0,
    writable: !0
}), "undefined" != typeof window && void 0 !== window.EventTarget && Object.setPrototypeOf(EventTarget.prototype, window.EventTarget.prototype);

class AbortSignal extends EventTarget {
    constructor(){
        throw super(), TypeError("AbortSignal cannot be constructed directly");
    }
    get aborted() {
        let aborted = abortedFlags.get(this);
        if ("boolean" != typeof aborted) throw TypeError(`Expected 'this' to be an 'AbortSignal' object, but got ${this === null ? "null" : typeof this}`);
        return aborted;
    }
}
defineEventAttribute(AbortSignal.prototype, "abort");
let abortedFlags = new WeakMap();
Object.defineProperties(AbortSignal.prototype, {
    aborted: {
        enumerable: !0
    }
}), "function" == typeof Symbol && "symbol" == typeof Symbol.toStringTag && Object.defineProperty(AbortSignal.prototype, Symbol.toStringTag, {
    configurable: !0,
    value: "AbortSignal"
});
class AbortController$1 {
    constructor(){
        signals.set(this, function() {
            let signal = Object.create(AbortSignal.prototype);
            return EventTarget.call(signal), abortedFlags.set(signal, !1), signal;
        }());
    }
    get signal() {
        return getSignal(this);
    }
    abort() {
        var signal;
        signal = getSignal(this), !1 === abortedFlags.get(signal) && (abortedFlags.set(signal, !0), signal.dispatchEvent({
            type: "abort"
        }));
    }
}
let signals = new WeakMap();
function getSignal(controller) {
    let signal = signals.get(controller);
    if (null == signal) throw TypeError(`Expected 'this' to be an 'AbortController' object, but got ${null === controller ? "null" : typeof controller}`);
    return signal;
}
Object.defineProperties(AbortController$1.prototype, {
    signal: {
        enumerable: !0
    },
    abort: {
        enumerable: !0
    }
}), "function" == typeof Symbol && "symbol" == typeof Symbol.toStringTag && Object.defineProperty(AbortController$1.prototype, Symbol.toStringTag, {
    configurable: !0,
    value: "AbortController"
});

let createIterator;
function encodePathSegment(segment) {
    return encodeURIComponent(segment).replace(/[!'()*]/g, (c)=>`%${c.charCodeAt(0).toString(16)}`);
}
createIterator = Symbol?.iterator && "function" == typeof [][Symbol.iterator] ? (items)=>items[Symbol.iterator]() : (items)=>({
        next: ()=>{
            let value = items.shift();
            return {
                done: void 0 === value,
                value: value
            };
        }
    });
class URLSearchParams {
    _entries;
    constructor(init){
        if (this._entries = {}, "string" == typeof init) {
            if ("" !== init) {
                let attribute;
                let attributes = (init = init.replace(/^\?/, "")).split("&");
                for(let i = 0; i < attributes.length; i++)attribute = attributes[i].split("="), this.append(decodeURIComponent(attribute[0]), attribute.length > 1 ? decodeURIComponent(attribute[1]) : "");
            }
        } else init instanceof URLSearchParams && init.forEach((value, name)=>{
            this.append(value, name);
        });
    }
    append(name, value) {
        value = value.toString(), name in this._entries ? this._entries[name].push(value) : this._entries[name] = [
            value
        ];
    }
    delete(name) {
        delete this._entries[name];
    }
    get(name) {
        return name in this._entries ? this._entries[name][0] : null;
    }
    getAll(name) {
        return name in this._entries ? this._entries[name].slice(0) : [];
    }
    has(name) {
        return name in this._entries;
    }
    set(name, value) {
        this._entries[name] = [
            value.toString()
        ];
    }
    forEach(callback) {
        let entries;
        for(let name in this._entries)if (this._entries.hasOwnProperty(name)) {
            entries = this._entries[name];
            for(let i = 0; i < entries.length; i++)callback.call(this, entries[i], name, this);
        }
    }
    keys() {
        let items = [];
        return this.forEach((value, name)=>{
            items.push(name);
        }), createIterator(items);
    }
    values() {
        let items = [];
        return this.forEach((value)=>{
            items.push(value);
        }), createIterator(items);
    }
    entries() {
        let items = [];
        return this.forEach((value, name)=>{
            items.push([
                value,
                name
            ]);
        }), createIterator(items);
    }
    toString() {
        let searchString = "";
        return this.forEach((value, name)=>{
            searchString.length > 0 && (searchString += "&"), searchString += encodeURIComponent(name) + "=" + encodeURIComponent(value);
        }), searchString;
    }
}
class URL {
    static patterns = {
        protocol: "(?:([^:/?#]+):)",
        authority: "(?://([^/?#]*))",
        path: "([^?#]*)",
        query: "(\\?[^#]*)",
        hash: "(#.*)",
        authentication: "(?:([^:]*)(?::([^@]*))?@)",
        hostname: "([^:]+)",
        port: "(?::(\\d+))"
    };
    static URLRegExp;
    static AuthorityRegExp;
    static init() {
        this.URLRegExp = RegExp("^" + this.patterns.protocol + "?" + this.patterns.authority + "?" + this.patterns.path + this.patterns.query + "?" + this.patterns.hash + "?"), this.AuthorityRegExp = RegExp("^" + this.patterns.authentication + "?" + this.patterns.hostname + this.patterns.port + "?$");
    }
    static parse(url) {
        let urlMatch = this.URLRegExp.exec(url);
        if (null !== urlMatch) {
            let authorityMatch = urlMatch[2] ? this.AuthorityRegExp.exec(urlMatch[2]) : [
                null,
                null,
                null,
                null,
                null
            ];
            if (null !== authorityMatch) return {
                protocol: urlMatch[1] || "",
                username: authorityMatch[1] || "",
                password: authorityMatch[2] || "",
                hostname: authorityMatch[3] || "",
                port: authorityMatch[4] || "",
                path: urlMatch[3] || "",
                query: urlMatch[4] || "",
                hash: urlMatch[5] || ""
            };
        }
        throw Error("Invalid URL");
    }
    _parts;
    constructor(url, base){
        let baseParts;
        try {
            baseParts = URL.parse(base);
        } catch (e) {
            throw Error("Invalid base URL");
        }
        let urlParts = URL.parse(url);
        urlParts.protocol ? this._parts = {
            ...urlParts
        } : this._parts = {
            protocol: baseParts.protocol,
            username: baseParts.username,
            password: baseParts.password,
            hostname: baseParts.hostname,
            port: baseParts.port,
            path: urlParts.path || baseParts.path,
            query: urlParts.query || baseParts.query,
            hash: urlParts.hash
        };
    }
    get hash() {
        return this._parts.hash;
    }
    set hash(value) {
        0 === (value = value.toString()).length ? this._parts.hash = "" : ("#" !== value.charAt(0) && (value = "#" + value), this._parts.hash = encodeURIComponent(value));
    }
    get host() {
        return this.hostname + (this.port ? ":" + this.port : "");
    }
    set host(value) {
        let url = new URL("http://" + (value = value.toString()));
        this._parts.hostname = url.hostname, this._parts.port = url.port;
    }
    get hostname() {
        return this._parts.hostname;
    }
    set hostname(value) {
        value = value.toString(), this._parts.hostname = encodeURIComponent(value);
    }
    get href() {
        let authentication = this.username || this.password ? this.username + (this.password ? ":" + this.password : "") + "@" : "";
        return this.protocol + "//" + authentication + this.host + this.pathname + this.search + this.hash;
    }
    set href(value) {
        let url = new URL(value = value.toString());
        this._parts = {
            ...url._parts
        };
    }
    get origin() {
        return this.protocol + "//" + this.host;
    }
    get password() {
        return this._parts.password;
    }
    set password(value) {
        value = value.toString(), this._parts.password = encodeURIComponent(value);
    }
    get pathname() {
        return this._parts.path ? this._parts.path : "/";
    }
    set pathname(value) {
        let chunks = value.toString().split("/").map(encodePathSegment);
        chunks[0] && chunks.unshift(""), this._parts.path = chunks.join("/");
    }
    get port() {
        return this._parts.port;
    }
    set port(value) {
        let port = parseInt(value);
        isNaN(port) ? this._parts.port = "0" : this._parts.port = Math.max(0, port % 65536).toString();
    }
    get protocol() {
        return this._parts.protocol + ":";
    }
    set protocol(value) {
        0 !== (value = value.toString()).length && (":" === value.charAt(value.length - 1) && (value = value.slice(0, -1)), this._parts.protocol = encodeURIComponent(value));
    }
    get search() {
        return this._parts.query;
    }
    set search(value) {
        "?" !== (value = value.toString()).charAt(0) && (value = "?" + value), this._parts.query = value;
    }
    get username() {
        return this._parts.username;
    }
    set username(value) {
        value = value.toString(), this._parts.username = encodeURIComponent(value);
    }
    get searchParams() {
        let searchParams = new URLSearchParams(this.search);
        return [
            "append",
            "delete",
            "set"
        ].forEach((methodName)=>{
            let method = searchParams[methodName];
            searchParams[methodName] = (...args)=>{
                method.apply(searchParams, args), this.search = searchParams.toString();
            };
        }), searchParams;
    }
    toString() {
        return this.href;
    }
}
URL.init();

let http = await import('@klaver/http'), CLIENT = new http.Client();
async function init$4(global) {
    Object.defineProperty(global, "fetch", {
        value: fetchImpl,
        configurable: !0,
        writable: !0
    }), Object.defineProperties(global, {
        AbortController: {
            value: AbortController$1
        },
        AbortSignal: {
            value: AbortSignal
        },
        URL: {
            value: URL
        },
        URLSearchParams: {
            value: URLSearchParams
        },
        Request: {
            value: class {
            }
        },
        Response: {
            value: class {
            }
        }
    });
}
function fetchImpl(input, init) {
    let opts = {
        headers: new http.Headers()
    };
    init?.headers && (init.headers, Array.isArray(init.headers) || Object.entries(init.headers)), init?.signal && (opts.cancel = new http.Cancel(), init.signal.onabort = opts.cancel.cancel.bind(opts.cancel));
    let req = new http.Request(input?.toString(), opts);
    return CLIENT.send(req);
}

var te, re, oe;
function e() {}
function t(e) {
    return "object" == typeof e && null !== e || "function" == typeof e;
}
function o(e, t) {
    try {
        Object.defineProperty(e, "name", {
            value: t,
            configurable: !0
        });
    } catch (e) {}
}
let n = Promise, a = Promise.resolve.bind(n), i = Promise.prototype.then, l = Promise.reject.bind(n);
function u(e) {
    return new n(e);
}
function c(e) {
    return u((t)=>t(e));
}
function f(e, t, r) {
    return i.call(e, t, r);
}
function b(e1, t, o) {
    f(f(e1, t, o), void 0, e);
}
function m(e, t) {
    b(e, void 0, t);
}
function p(e1) {
    f(e1, void 0, e);
}
let y = (e)=>{
    if ("function" == typeof queueMicrotask) y = queueMicrotask;
    else {
        let e = c(void 0);
        y = (t)=>f(e, t);
    }
    return y(e);
};
function S(e, t, r) {
    if ("function" != typeof e) throw TypeError("Argument is not a function");
    return Function.prototype.apply.call(e, t, r);
}
function g(e, t, r) {
    try {
        return c(S(e, t, r));
    } catch (e) {
        return l(e);
    }
}
class v {
    constructor(){
        this._cursor = 0, this._size = 0, this._front = {
            _elements: [],
            _next: void 0
        }, this._back = this._front, this._cursor = 0, this._size = 0;
    }
    get length() {
        return this._size;
    }
    push(e) {
        let t = this._back, r = t;
        16383 === t._elements.length && (r = {
            _elements: [],
            _next: void 0
        }), t._elements.push(e), r !== t && (this._back = r, t._next = r), ++this._size;
    }
    shift() {
        let e = this._front, t = e, r = this._cursor, o = r + 1, n = e._elements, a = n[r];
        return 16384 === o && (t = e._next, o = 0), --this._size, this._cursor = o, e !== t && (this._front = t), n[r] = void 0, a;
    }
    forEach(e) {
        let t = this._cursor, r = this._front, o = r._elements;
        for(; !(t === o.length && void 0 === r._next || t === o.length && (o = (r = r._next)._elements, t = 0, 0 === o.length));)e(o[t]), ++t;
    }
    peek() {
        let e = this._front, t = this._cursor;
        return e._elements[t];
    }
}
let w = Symbol("[[AbortSteps]]"), R = Symbol("[[ErrorSteps]]"), T = Symbol("[[CancelSteps]]"), C = Symbol("[[PullSteps]]"), P = Symbol("[[ReleaseSteps]]");
function q(e, t) {
    var t1;
    e._ownerReadableStream = t, t._reader = e, "readable" === t._state ? B(e) : "closed" === t._state ? (B(e), A(e)) : (t1 = t._storedError, B(e), j(e, t1));
}
function E(e, t) {
    return Or(e._ownerReadableStream, t);
}
function W(e) {
    var t;
    let t1 = e._ownerReadableStream;
    "readable" === t1._state ? j(e, TypeError("Reader was released and can no longer be used to monitor the stream's closedness")) : (t = TypeError("Reader was released and can no longer be used to monitor the stream's closedness"), B(e), j(e, t)), t1._readableStreamController[P](), t1._reader = void 0, e._ownerReadableStream = void 0;
}
function O(e) {
    return TypeError("Cannot " + e + " a stream using a released reader");
}
function B(e) {
    e._closedPromise = u((t, r)=>{
        e._closedPromise_resolve = t, e._closedPromise_reject = r;
    });
}
function j(e, t) {
    void 0 !== e._closedPromise_reject && (p(e._closedPromise), e._closedPromise_reject(t), e._closedPromise_resolve = void 0, e._closedPromise_reject = void 0);
}
function A(e) {
    void 0 !== e._closedPromise_resolve && (e._closedPromise_resolve(void 0), e._closedPromise_resolve = void 0, e._closedPromise_reject = void 0);
}
let z = Number.isFinite || function(e) {
    return "number" == typeof e && isFinite(e);
}, D = Math.trunc || function(e) {
    return e < 0 ? Math.ceil(e) : Math.floor(e);
};
function L(e, t) {
    if (void 0 !== e && "object" != typeof e && "function" != typeof e) throw TypeError(`${t} is not an object.`);
}
function F(e, t) {
    if ("function" != typeof e) throw TypeError(`${t} is not a function.`);
}
function I(e, t) {
    if (!("object" == typeof e && null !== e || "function" == typeof e)) throw TypeError(`${t} is not an object.`);
}
function $(e, t, r) {
    if (void 0 === e) throw TypeError(`Parameter ${t} is required in '${r}'.`);
}
function M(e, t, r) {
    if (void 0 === e) throw TypeError(`${t} is required in '${r}'.`);
}
function Y(e) {
    return Number(e);
}
function Q(e, t) {
    var e1, e2;
    let r = Number.MAX_SAFE_INTEGER, o = Number(e);
    if (!z(o = 0 === (e1 = o) ? 0 : e1)) throw TypeError(`${t} is not a finite number`);
    if ((o = 0 === (e2 = D(o)) ? 0 : e2) < 0 || o > r) throw TypeError(`${t} is outside the accepted range of 0 to ${r}, inclusive`);
    return z(o) && 0 !== o ? o : 0;
}
function N(e, t) {
    if (!Er(e)) throw TypeError(`${t} is not a ReadableStream.`);
}
function H(e) {
    return new ReadableStreamDefaultReader(e);
}
function V(e, t) {
    e._reader._readRequests.push(t);
}
function U(e, t, r) {
    let o = e._reader._readRequests.shift();
    r ? o._closeSteps() : o._chunkSteps(t);
}
function G(e) {
    return e._reader._readRequests.length;
}
function X(e) {
    let t = e._reader;
    return void 0 !== t && !!J(t);
}
class ReadableStreamDefaultReader {
    constructor(e){
        if ($(e, 1, "ReadableStreamDefaultReader"), N(e, "First parameter"), Wr(e)) throw TypeError("This stream has already been locked for exclusive reading by another reader");
        q(this, e), this._readRequests = new v;
    }
    get closed() {
        return J(this) ? this._closedPromise : l(ee("closed"));
    }
    cancel(e) {
        return J(this) ? void 0 === this._ownerReadableStream ? l(O("cancel")) : E(this, e) : l(ee("cancel"));
    }
    read() {
        let e, t;
        if (!J(this)) return l(ee("read"));
        if (void 0 === this._ownerReadableStream) return l(O("read from"));
        let r = u((r, o)=>{
            e = r, t = o;
        });
        return K(this, {
            _chunkSteps: (t)=>e({
                    value: t,
                    done: !1
                }),
            _closeSteps: ()=>e({
                    value: void 0,
                    done: !0
                }),
            _errorSteps: (e)=>t(e)
        }), r;
    }
    releaseLock() {
        if (!J(this)) throw ee("releaseLock");
        void 0 !== this._ownerReadableStream && (W(this), Z(this, TypeError("Reader was released")));
    }
}
function J(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_readRequests") && e instanceof ReadableStreamDefaultReader;
}
function K(e, t) {
    let r = e._ownerReadableStream;
    r._disturbed = !0, "closed" === r._state ? t._closeSteps() : "errored" === r._state ? t._errorSteps(r._storedError) : r._readableStreamController[C](t);
}
function Z(e, t) {
    let r = e._readRequests;
    e._readRequests = new v, r.forEach((e)=>{
        e._errorSteps(t);
    });
}
function ee(e) {
    return TypeError(`ReadableStreamDefaultReader.prototype.${e} can only be used on a ReadableStreamDefaultReader`);
}
function ne(e) {
    return e.slice();
}
function ae(e, t, r, o, n) {
    new Uint8Array(e).set(new Uint8Array(r, o, n), t);
}
Object.defineProperties(ReadableStreamDefaultReader.prototype, {
    cancel: {
        enumerable: !0
    },
    read: {
        enumerable: !0
    },
    releaseLock: {
        enumerable: !0
    },
    closed: {
        enumerable: !0
    }
}), o(ReadableStreamDefaultReader.prototype.cancel, "cancel"), o(ReadableStreamDefaultReader.prototype.read, "read"), o(ReadableStreamDefaultReader.prototype.releaseLock, "releaseLock"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamDefaultReader.prototype, Symbol.toStringTag, {
    value: "ReadableStreamDefaultReader",
    configurable: !0
});
let ie = (e)=>(ie = "function" == typeof e.transfer ? (e)=>e.transfer() : "function" == typeof structuredClone ? (e)=>structuredClone(e, {
            transfer: [
                e
            ]
        }) : (e)=>e)(e), le = (e)=>(le = "boolean" == typeof e.detached ? (e)=>e.detached : (e)=>0 === e.byteLength)(e);
function se(e, t, r) {
    if (e.slice) return e.slice(t, r);
    let o = r - t, n = new ArrayBuffer(o);
    return ae(n, 0, e, t, o), n;
}
function ue(e, t) {
    let r = e[t];
    if (null != r) {
        if ("function" != typeof r) throw TypeError(`${String(t)} is not a function`);
        return r;
    }
}
function ce(e) {
    try {
        let t = e.done, r = e.value;
        return f(a(r), (e)=>({
                done: t,
                value: e
            }));
    } catch (e) {
        return l(e);
    }
}
let de = null !== (oe = null !== (te = Symbol.asyncIterator) && void 0 !== te ? te : null === (re = Symbol.for) || void 0 === re ? void 0 : re.call(Symbol, "Symbol.asyncIterator")) && void 0 !== oe ? oe : "@@asyncIterator";
function be(e) {
    let r = S(e.nextMethod, e.iterator, []);
    if (!t(r)) throw TypeError("The iterator.next() method must return an object");
    return r;
}
class he {
    constructor(e, t){
        this._ongoingPromise = void 0, this._isFinished = !1, this._reader = e, this._preventCancel = t;
    }
    next() {
        let e = ()=>this._nextSteps();
        return this._ongoingPromise = this._ongoingPromise ? f(this._ongoingPromise, e, e) : e(), this._ongoingPromise;
    }
    return(e) {
        let t = ()=>this._returnSteps(e);
        return this._ongoingPromise ? f(this._ongoingPromise, t, t) : t();
    }
    _nextSteps() {
        let t, r;
        if (this._isFinished) return Promise.resolve({
            value: void 0,
            done: !0
        });
        let e = this._reader, o = u((e, o)=>{
            t = e, r = o;
        });
        return K(e, {
            _chunkSteps: (e)=>{
                this._ongoingPromise = void 0, y(()=>t({
                        value: e,
                        done: !1
                    }));
            },
            _closeSteps: ()=>{
                this._ongoingPromise = void 0, this._isFinished = !0, W(e), t({
                    value: void 0,
                    done: !0
                });
            },
            _errorSteps: (t)=>{
                this._ongoingPromise = void 0, this._isFinished = !0, W(e), r(t);
            }
        }), o;
    }
    _returnSteps(e) {
        if (this._isFinished) return Promise.resolve({
            value: e,
            done: !0
        });
        this._isFinished = !0;
        let t = this._reader;
        if (!this._preventCancel) {
            let r = E(t, e);
            return W(t), f(r, ()=>({
                    value: e,
                    done: !0
                }), void 0);
        }
        return W(t), c({
            value: e,
            done: !0
        });
    }
}
let me = {
    next () {
        return _e(this) ? this._asyncIteratorImpl.next() : l(pe("next"));
    },
    return (e) {
        return _e(this) ? this._asyncIteratorImpl.return(e) : l(pe("return"));
    },
    [de] () {
        return this;
    }
};
function _e(e) {
    if (!t(e) || !Object.prototype.hasOwnProperty.call(e, "_asyncIteratorImpl")) return !1;
    try {
        return e._asyncIteratorImpl instanceof he;
    } catch (e) {
        return !1;
    }
}
function pe(e) {
    return TypeError(`ReadableStreamAsyncIterator.${e} can only be used on a ReadableSteamAsyncIterator`);
}
Object.defineProperty(me, de, {
    enumerable: !1
});
let ye = Number.isNaN || function(e) {
    return e != e;
};
function Se(e) {
    return new Uint8Array(se(e.buffer, e.byteOffset, e.byteOffset + e.byteLength));
}
function ge(e) {
    let t = e._queue.shift();
    return e._queueTotalSize -= t.size, e._queueTotalSize < 0 && (e._queueTotalSize = 0), t.value;
}
function ve(e, t, r) {
    if ("number" != typeof r || ye(r) || r < 0 || r === 1 / 0) throw RangeError("Size must be a finite, non-NaN, non-negative number.");
    e._queue.push({
        value: t,
        size: r
    }), e._queueTotalSize += r;
}
function we(e) {
    e._queue = new v, e._queueTotalSize = 0;
}
function Re(e) {
    return e === DataView;
}
class ReadableStreamBYOBRequest {
    constructor(){
        throw TypeError("Illegal constructor");
    }
    get view() {
        if (!Ce(this)) throw Je("view");
        return this._view;
    }
    respond(e) {
        if (!Ce(this)) throw Je("respond");
        if ($(e, 1, "respond"), e = Q(e, "First parameter"), void 0 === this._associatedReadableByteStreamController) throw TypeError("This BYOB request has been invalidated");
        if (le(this._view.buffer)) throw TypeError("The BYOB request's buffer has been detached and so cannot be used as a response");
        Ue(this._associatedReadableByteStreamController, e);
    }
    respondWithNewView(e) {
        if (!Ce(this)) throw Je("respondWithNewView");
        if ($(e, 1, "respondWithNewView"), !ArrayBuffer.isView(e)) throw TypeError("You can only respond with array buffer views");
        if (void 0 === this._associatedReadableByteStreamController) throw TypeError("This BYOB request has been invalidated");
        if (le(e.buffer)) throw TypeError("The given view's buffer has been detached and so cannot be used as a response");
        Ge(this._associatedReadableByteStreamController, e);
    }
}
Object.defineProperties(ReadableStreamBYOBRequest.prototype, {
    respond: {
        enumerable: !0
    },
    respondWithNewView: {
        enumerable: !0
    },
    view: {
        enumerable: !0
    }
}), o(ReadableStreamBYOBRequest.prototype.respond, "respond"), o(ReadableStreamBYOBRequest.prototype.respondWithNewView, "respondWithNewView"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamBYOBRequest.prototype, Symbol.toStringTag, {
    value: "ReadableStreamBYOBRequest",
    configurable: !0
});
class ReadableByteStreamController {
    constructor(){
        throw TypeError("Illegal constructor");
    }
    get byobRequest() {
        if (!Te(this)) throw Ke("byobRequest");
        return He(this);
    }
    get desiredSize() {
        if (!Te(this)) throw Ke("desiredSize");
        return Ve(this);
    }
    close() {
        if (!Te(this)) throw Ke("close");
        if (this._closeRequested) throw TypeError("The stream has already been closed; do not close it again!");
        let e = this._controlledReadableByteStream._state;
        if ("readable" !== e) throw TypeError(`The stream (in ${e} state) is not in the readable state and cannot be closed`);
        Ye(this);
    }
    enqueue(e) {
        if (!Te(this)) throw Ke("enqueue");
        if ($(e, 1, "enqueue"), !ArrayBuffer.isView(e)) throw TypeError("chunk must be an array buffer view");
        if (0 === e.byteLength) throw TypeError("chunk must have non-zero byteLength");
        if (0 === e.buffer.byteLength) throw TypeError("chunk's buffer must have non-zero byteLength");
        if (this._closeRequested) throw TypeError("stream is closed or draining");
        let t = this._controlledReadableByteStream._state;
        if ("readable" !== t) throw TypeError(`The stream (in ${t} state) is not in the readable state and cannot be enqueued to`);
        xe(this, e);
    }
    error(e) {
        if (!Te(this)) throw Ke("error");
        Qe(this, e);
    }
    [T](e) {
        qe(this), we(this);
        let t = this._cancelAlgorithm(e);
        return Me(this), t;
    }
    [C](e) {
        let t = this._controlledReadableByteStream;
        if (this._queueTotalSize > 0) return void Ne(this, e);
        let r = this._autoAllocateChunkSize;
        if (void 0 !== r) {
            let t;
            try {
                t = new ArrayBuffer(r);
            } catch (t) {
                return void e._errorSteps(t);
            }
            let o = {
                buffer: t,
                bufferByteLength: r,
                byteOffset: 0,
                byteLength: r,
                bytesFilled: 0,
                minimumFill: 1,
                elementSize: 1,
                viewConstructor: Uint8Array,
                readerType: "default"
            };
            this._pendingPullIntos.push(o);
        }
        V(t, e), Pe(this);
    }
    [P]() {
        if (this._pendingPullIntos.length > 0) {
            let e = this._pendingPullIntos.peek();
            e.readerType = "none", this._pendingPullIntos = new v, this._pendingPullIntos.push(e);
        }
    }
}
function Te(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_controlledReadableByteStream") && e instanceof ReadableByteStreamController;
}
function Ce(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_associatedReadableByteStreamController") && e instanceof ReadableStreamBYOBRequest;
}
function Pe(e) {
    if (function(e) {
        let t = e._controlledReadableByteStream;
        return "readable" === t._state && !e._closeRequested && !!e._started && !!(X(t) && G(t) > 0 || ot(t) && rt(t) > 0 || Ve(e) > 0);
    }(e)) {
        if (e._pulling) return void (e._pullAgain = !0);
        e._pulling = !0, b(e._pullAlgorithm(), ()=>(e._pulling = !1, e._pullAgain && (e._pullAgain = !1, Pe(e)), null), (t)=>(Qe(e, t), null));
    }
}
function qe(e) {
    De(e), e._pendingPullIntos = new v;
}
function Ee(e, t) {
    let r = !1;
    "closed" === e._state && (r = !0);
    let o = We(t);
    "default" === t.readerType ? U(e, o, r) : function(e, t, r) {
        let n = e._reader._readIntoRequests.shift();
        r ? n._closeSteps(t) : n._chunkSteps(t);
    }(e, o, r);
}
function We(e) {
    let t = e.bytesFilled, r = e.elementSize;
    return new e.viewConstructor(e.buffer, e.byteOffset, t / r);
}
function Oe(e, t, r, o) {
    e._queue.push({
        buffer: t,
        byteOffset: r,
        byteLength: o
    }), e._queueTotalSize += o;
}
function Be(e, t, r, o) {
    let n;
    try {
        n = se(t, r, r + o);
    } catch (t) {
        throw Qe(e, t), t;
    }
    Oe(e, n, 0, o);
}
function ke(e, t) {
    t.bytesFilled > 0 && Be(e, t.buffer, t.byteOffset, t.bytesFilled), $e(e);
}
function je(e, t) {
    let r = Math.min(e._queueTotalSize, t.byteLength - t.bytesFilled), o = t.bytesFilled + r, n = r, a = !1, i = o - o % t.elementSize;
    i >= t.minimumFill && (n = i - t.bytesFilled, a = !0);
    let l = e._queue;
    for(; n > 0;){
        let r = l.peek(), o = Math.min(n, r.byteLength), a = t.byteOffset + t.bytesFilled;
        ae(t.buffer, a, r.buffer, r.byteOffset, o), r.byteLength === o ? l.shift() : (r.byteOffset += o, r.byteLength -= o), e._queueTotalSize -= o, Ae(e, o, t), n -= o;
    }
    return a;
}
function Ae(e, t, r) {
    r.bytesFilled += t;
}
function ze(e) {
    0 === e._queueTotalSize && e._closeRequested ? (Me(e), Br(e._controlledReadableByteStream)) : Pe(e);
}
function De(e) {
    null !== e._byobRequest && (e._byobRequest._associatedReadableByteStreamController = void 0, e._byobRequest._view = null, e._byobRequest = null);
}
function Le(e) {
    for(; e._pendingPullIntos.length > 0;){
        if (0 === e._queueTotalSize) return;
        let t = e._pendingPullIntos.peek();
        je(e, t) && ($e(e), Ee(e._controlledReadableByteStream, t));
    }
}
function Ie(e, t) {
    let r = e._pendingPullIntos.peek();
    De(e), "closed" === e._controlledReadableByteStream._state ? function(e, t) {
        "none" === t.readerType && $e(e);
        let r = e._controlledReadableByteStream;
        if (ot(r)) for(; rt(r) > 0;)Ee(r, $e(e));
    }(e, r) : function(e, t, r) {
        if (Ae(0, t, r), "none" === r.readerType) return ke(e, r), void Le(e);
        if (r.bytesFilled < r.minimumFill) return;
        $e(e);
        let o = r.bytesFilled % r.elementSize;
        if (o > 0) {
            let t = r.byteOffset + r.bytesFilled;
            Be(e, r.buffer, t - o, o);
        }
        r.bytesFilled -= o, Ee(e._controlledReadableByteStream, r), Le(e);
    }(e, t, r), Pe(e);
}
function $e(e) {
    return e._pendingPullIntos.shift();
}
function Me(e) {
    e._pullAlgorithm = void 0, e._cancelAlgorithm = void 0;
}
function Ye(e) {
    let t = e._controlledReadableByteStream;
    if (!e._closeRequested && "readable" === t._state) {
        if (e._queueTotalSize > 0) e._closeRequested = !0;
        else {
            if (e._pendingPullIntos.length > 0) {
                let t = e._pendingPullIntos.peek();
                if (t.bytesFilled % t.elementSize != 0) {
                    let t = TypeError("Insufficient bytes to fill elements in the given buffer");
                    throw Qe(e, t), t;
                }
            }
            Me(e), Br(t);
        }
    }
}
function xe(e, t) {
    let r = e._controlledReadableByteStream;
    if (e._closeRequested || "readable" !== r._state) return;
    let { buffer: o, byteOffset: n, byteLength: a } = t;
    if (le(o)) throw TypeError("chunk's buffer is detached and so cannot be enqueued");
    let i = ie(o);
    if (e._pendingPullIntos.length > 0) {
        let t = e._pendingPullIntos.peek();
        if (le(t.buffer)) throw TypeError("The BYOB request's buffer has been detached and so cannot be filled with an enqueued chunk");
        De(e), t.buffer = ie(t.buffer), "none" === t.readerType && ke(e, t);
    }
    X(r) ? (function(e) {
        let t = e._controlledReadableByteStream._reader;
        for(; t._readRequests.length > 0;){
            if (0 === e._queueTotalSize) return;
            Ne(e, t._readRequests.shift());
        }
    }(e), 0 === G(r)) ? Oe(e, i, n, a) : (e._pendingPullIntos.length > 0 && $e(e), U(r, new Uint8Array(i, n, a), !1)) : ot(r) ? (Oe(e, i, n, a), Le(e)) : Oe(e, i, n, a), Pe(e);
}
function Qe(e, t) {
    let r = e._controlledReadableByteStream;
    "readable" === r._state && (qe(e), we(e), Me(e), kr(r, t));
}
function Ne(e, t) {
    let r = e._queue.shift();
    e._queueTotalSize -= r.byteLength, ze(e);
    let o = new Uint8Array(r.buffer, r.byteOffset, r.byteLength);
    t._chunkSteps(o);
}
function He(e) {
    if (null === e._byobRequest && e._pendingPullIntos.length > 0) {
        let t = e._pendingPullIntos.peek(), r = new Uint8Array(t.buffer, t.byteOffset + t.bytesFilled, t.byteLength - t.bytesFilled), o = Object.create(ReadableStreamBYOBRequest.prototype);
        o._associatedReadableByteStreamController = e, o._view = r, e._byobRequest = o;
    }
    return e._byobRequest;
}
function Ve(e) {
    let t = e._controlledReadableByteStream._state;
    return "errored" === t ? null : "closed" === t ? 0 : e._strategyHWM - e._queueTotalSize;
}
function Ue(e, t) {
    let r = e._pendingPullIntos.peek();
    if ("closed" === e._controlledReadableByteStream._state) {
        if (0 !== t) throw TypeError("bytesWritten must be 0 when calling respond() on a closed stream");
    } else {
        if (0 === t) throw TypeError("bytesWritten must be greater than 0 when calling respond() on a readable stream");
        if (r.bytesFilled + t > r.byteLength) throw RangeError("bytesWritten out of range");
    }
    r.buffer = ie(r.buffer), Ie(e, t);
}
function Ge(e, t) {
    let r = e._pendingPullIntos.peek();
    if ("closed" === e._controlledReadableByteStream._state) {
        if (0 !== t.byteLength) throw TypeError("The view's length must be 0 when calling respondWithNewView() on a closed stream");
    } else if (0 === t.byteLength) throw TypeError("The view's length must be greater than 0 when calling respondWithNewView() on a readable stream");
    if (r.byteOffset + r.bytesFilled !== t.byteOffset) throw RangeError("The region specified by view does not match byobRequest");
    if (r.bufferByteLength !== t.buffer.byteLength) throw RangeError("The buffer of view has different capacity than byobRequest");
    if (r.bytesFilled + t.byteLength > r.byteLength) throw RangeError("The region specified by view is larger than byobRequest");
    let o = t.byteLength;
    r.buffer = ie(t.buffer), Ie(e, o);
}
function Xe(e, t, r, o, n, a, i) {
    t._controlledReadableByteStream = e, t._pullAgain = !1, t._pulling = !1, t._byobRequest = null, t._queue = t._queueTotalSize = void 0, we(t), t._closeRequested = !1, t._started = !1, t._strategyHWM = a, t._pullAlgorithm = o, t._cancelAlgorithm = n, t._autoAllocateChunkSize = i, t._pendingPullIntos = new v, e._readableStreamController = t, b(c(r()), ()=>(t._started = !0, Pe(t), null), (e)=>(Qe(t, e), null));
}
function Je(e) {
    return TypeError(`ReadableStreamBYOBRequest.prototype.${e} can only be used on a ReadableStreamBYOBRequest`);
}
function Ke(e) {
    return TypeError(`ReadableByteStreamController.prototype.${e} can only be used on a ReadableByteStreamController`);
}
function tt(e, t) {
    e._reader._readIntoRequests.push(t);
}
function rt(e) {
    return e._reader._readIntoRequests.length;
}
function ot(e) {
    let t = e._reader;
    return void 0 !== t && !!nt(t);
}
Object.defineProperties(ReadableByteStreamController.prototype, {
    close: {
        enumerable: !0
    },
    enqueue: {
        enumerable: !0
    },
    error: {
        enumerable: !0
    },
    byobRequest: {
        enumerable: !0
    },
    desiredSize: {
        enumerable: !0
    }
}), o(ReadableByteStreamController.prototype.close, "close"), o(ReadableByteStreamController.prototype.enqueue, "enqueue"), o(ReadableByteStreamController.prototype.error, "error"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableByteStreamController.prototype, Symbol.toStringTag, {
    value: "ReadableByteStreamController",
    configurable: !0
});
class ReadableStreamBYOBReader {
    constructor(e){
        if ($(e, 1, "ReadableStreamBYOBReader"), N(e, "First parameter"), Wr(e)) throw TypeError("This stream has already been locked for exclusive reading by another reader");
        if (!Te(e._readableStreamController)) throw TypeError("Cannot construct a ReadableStreamBYOBReader for a stream not constructed with a byte source");
        q(this, e), this._readIntoRequests = new v;
    }
    get closed() {
        return nt(this) ? this._closedPromise : l(lt("closed"));
    }
    cancel(e) {
        return nt(this) ? void 0 === this._ownerReadableStream ? l(O("cancel")) : E(this, e) : l(lt("cancel"));
    }
    read(e, t = {}) {
        let r, n, a;
        if (!nt(this)) return l(lt("read"));
        if (!ArrayBuffer.isView(e)) return l(TypeError("view must be an array buffer view"));
        if (0 === e.byteLength) return l(TypeError("view must have non-zero byteLength"));
        if (0 === e.buffer.byteLength) return l(TypeError("view's buffer must have non-zero byteLength"));
        if (le(e.buffer)) return l(TypeError("view's buffer has been detached"));
        try {
            var t1, r1;
            t1 = "options", L(t, t1), r = {
                min: Q(null !== (r1 = null == t ? void 0 : t.min) && void 0 !== r1 ? r1 : 1, `${t1} has member 'min' that`)
            };
        } catch (e) {
            return l(e);
        }
        let o = r.min;
        if (0 === o) return l(TypeError("options.min must be greater than 0"));
        if (Re(e.constructor)) {
            if (o > e.byteLength) return l(RangeError("options.min must be less than or equal to view's byteLength"));
        } else if (o > e.length) return l(RangeError("options.min must be less than or equal to view's length"));
        if (void 0 === this._ownerReadableStream) return l(O("read from"));
        let i = u((e, t)=>{
            n = e, a = t;
        });
        return at(this, e, o, {
            _chunkSteps: (e)=>n({
                    value: e,
                    done: !1
                }),
            _closeSteps: (e)=>n({
                    value: e,
                    done: !0
                }),
            _errorSteps: (e)=>a(e)
        }), i;
    }
    releaseLock() {
        if (!nt(this)) throw lt("releaseLock");
        void 0 !== this._ownerReadableStream && (W(this), it(this, TypeError("Reader was released")));
    }
}
function nt(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_readIntoRequests") && e instanceof ReadableStreamBYOBReader;
}
function at(e, t, r, o) {
    let n = e._ownerReadableStream;
    n._disturbed = !0, "errored" === n._state ? o._errorSteps(n._storedError) : function(e, t, r, o) {
        let c;
        let n = e._controlledReadableByteStream, a = t.constructor, i = Re(a) ? 1 : a.BYTES_PER_ELEMENT, { byteOffset: l, byteLength: s } = t;
        try {
            c = ie(t.buffer);
        } catch (e) {
            return void o._errorSteps(e);
        }
        let d = {
            buffer: c,
            bufferByteLength: c.byteLength,
            byteOffset: l,
            byteLength: s,
            bytesFilled: 0,
            minimumFill: r * i,
            elementSize: i,
            viewConstructor: a,
            readerType: "byob"
        };
        if (e._pendingPullIntos.length > 0) return e._pendingPullIntos.push(d), void tt(n, o);
        if ("closed" !== n._state) {
            if (e._queueTotalSize > 0) {
                if (je(e, d)) {
                    let t = We(d);
                    return ze(e), void o._chunkSteps(t);
                }
                if (e._closeRequested) {
                    let t = TypeError("Insufficient bytes to fill elements in the given buffer");
                    return Qe(e, t), void o._errorSteps(t);
                }
            }
            e._pendingPullIntos.push(d), tt(n, o), Pe(e);
        } else {
            let e = new a(d.buffer, d.byteOffset, 0);
            o._closeSteps(e);
        }
    }(n._readableStreamController, t, r, o);
}
function it(e, t) {
    let r = e._readIntoRequests;
    e._readIntoRequests = new v, r.forEach((e)=>{
        e._errorSteps(t);
    });
}
function lt(e) {
    return TypeError(`ReadableStreamBYOBReader.prototype.${e} can only be used on a ReadableStreamBYOBReader`);
}
function st(e, t) {
    let { highWaterMark: r } = e;
    if (void 0 === r) return t;
    if (ye(r) || r < 0) throw RangeError("Invalid highWaterMark");
    return r;
}
function ut(e) {
    let { size: t } = e;
    return t || (()=>1);
}
function ct(e, t) {
    L(e, t);
    let r = null == e ? void 0 : e.highWaterMark, o = null == e ? void 0 : e.size;
    return {
        highWaterMark: void 0 === r ? void 0 : Y(r),
        size: void 0 === o ? void 0 : (F(o, `${t} has member 'size' that`), (t)=>Y(o(t)))
    };
}
function _t(e, t) {
    if (!gt(e)) throw TypeError(`${t} is not a WritableStream.`);
}
Object.defineProperties(ReadableStreamBYOBReader.prototype, {
    cancel: {
        enumerable: !0
    },
    read: {
        enumerable: !0
    },
    releaseLock: {
        enumerable: !0
    },
    closed: {
        enumerable: !0
    }
}), o(ReadableStreamBYOBReader.prototype.cancel, "cancel"), o(ReadableStreamBYOBReader.prototype.read, "read"), o(ReadableStreamBYOBReader.prototype.releaseLock, "releaseLock"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamBYOBReader.prototype, Symbol.toStringTag, {
    value: "ReadableStreamBYOBReader",
    configurable: !0
});
let pt = "function" == typeof AbortController;
class WritableStream {
    constructor(e = {}, t = {}){
        void 0 === e ? e = null : I(e, "First parameter");
        let r = ct(t, "Second parameter"), o = function(e, t) {
            L(e, t);
            let r = null == e ? void 0 : e.abort, o = null == e ? void 0 : e.close, n = null == e ? void 0 : e.start, a = null == e ? void 0 : e.type, i = null == e ? void 0 : e.write;
            return {
                abort: void 0 === r ? void 0 : (F(r, `${t} has member 'abort' that`), (r1)=>g(r, e, [
                        r1
                    ])),
                close: void 0 === o ? void 0 : (F(o, `${t} has member 'close' that`), ()=>g(o, e, [])),
                start: void 0 === n ? void 0 : (F(n, `${t} has member 'start' that`), (r)=>S(n, e, [
                        r
                    ])),
                write: void 0 === i ? void 0 : (F(i, `${t} has member 'write' that`), (r, o)=>g(i, e, [
                        r,
                        o
                    ])),
                type: a
            };
        }(e, "First parameter");
        if (St(this), void 0 !== o.type) throw RangeError("Invalid type is specified");
        let n = ut(r);
        !function(e, t, r, o) {
            let a, i;
            let n = Object.create(WritableStreamDefaultController.prototype);
            a = void 0 !== t.start ? ()=>t.start(n) : ()=>{}, i = void 0 !== t.write ? (e)=>t.write(e, n) : ()=>c(void 0), Ft(e, n, a, i, void 0 !== t.close ? ()=>t.close() : ()=>c(void 0), void 0 !== t.abort ? (e)=>t.abort(e) : ()=>c(void 0), r, o);
        }(this, o, st(r, 1), n);
    }
    get locked() {
        if (!gt(this)) throw Nt("locked");
        return vt(this);
    }
    abort(e) {
        return gt(this) ? vt(this) ? l(TypeError("Cannot abort a stream that already has a writer")) : wt(this, e) : l(Nt("abort"));
    }
    close() {
        return gt(this) ? vt(this) ? l(TypeError("Cannot close a stream that already has a writer")) : qt(this) ? l(TypeError("Cannot close an already-closing stream")) : Rt(this) : l(Nt("close"));
    }
    getWriter() {
        if (!gt(this)) throw Nt("getWriter");
        return new WritableStreamDefaultWriter(this);
    }
}
function St(e) {
    e._state = "writable", e._storedError = void 0, e._writer = void 0, e._writableStreamController = void 0, e._writeRequests = new v, e._inFlightWriteRequest = void 0, e._closeRequest = void 0, e._inFlightCloseRequest = void 0, e._pendingAbortRequest = void 0, e._backpressure = !1;
}
function gt(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_writableStreamController") && e instanceof WritableStream;
}
function vt(e) {
    return void 0 !== e._writer;
}
function wt(e, t) {
    var r;
    if ("closed" === e._state || "errored" === e._state) return c(void 0);
    e._writableStreamController._abortReason = t, null === (r = e._writableStreamController._abortController) || void 0 === r || r.abort(t);
    let o = e._state;
    if ("closed" === o || "errored" === o) return c(void 0);
    if (void 0 !== e._pendingAbortRequest) return e._pendingAbortRequest._promise;
    let n = !1;
    "erroring" === o && (n = !0, t = void 0);
    let a = u((r, o)=>{
        e._pendingAbortRequest = {
            _promise: void 0,
            _resolve: r,
            _reject: o,
            _reason: t,
            _wasAlreadyErroring: n
        };
    });
    return e._pendingAbortRequest._promise = a, n || Ct(e, t), a;
}
function Rt(e) {
    var n;
    let t = e._state;
    if ("closed" === t || "errored" === t) return l(TypeError(`The stream (in ${t} state) is not in the writable state and cannot be closed`));
    let r = u((t, r)=>{
        e._closeRequest = {
            _resolve: t,
            _reject: r
        };
    }), o = e._writer;
    return void 0 !== o && e._backpressure && "writable" === t && or(o), ve(n = e._writableStreamController, Dt, 0), Mt(n), r;
}
function Tt(e, t) {
    "writable" !== e._state ? Pt(e) : Ct(e, t);
}
function Ct(e, t) {
    let r = e._writableStreamController;
    e._state = "erroring", e._storedError = t;
    let o = e._writer;
    void 0 !== o && jt(o, t), !(void 0 !== e._inFlightWriteRequest || void 0 !== e._inFlightCloseRequest) && r._started && Pt(e);
}
function Pt(e) {
    e._state = "errored", e._writableStreamController[R]();
    let t = e._storedError;
    if (e._writeRequests.forEach((e)=>{
        e._reject(t);
    }), e._writeRequests = new v, void 0 === e._pendingAbortRequest) return void Et(e);
    let r = e._pendingAbortRequest;
    if (e._pendingAbortRequest = void 0, r._wasAlreadyErroring) return r._reject(t), void Et(e);
    b(e._writableStreamController[w](r._reason), ()=>(r._resolve(), Et(e), null), (t)=>(r._reject(t), Et(e), null));
}
function qt(e) {
    return void 0 !== e._closeRequest || void 0 !== e._inFlightCloseRequest;
}
function Et(e) {
    void 0 !== e._closeRequest && (e._closeRequest._reject(e._storedError), e._closeRequest = void 0);
    let t = e._writer;
    void 0 !== t && Jt(t, e._storedError);
}
function Wt(e, t) {
    let r = e._writer;
    void 0 !== r && t !== e._backpressure && (t ? Zt(r) : or(r)), e._backpressure = t;
}
Object.defineProperties(WritableStream.prototype, {
    abort: {
        enumerable: !0
    },
    close: {
        enumerable: !0
    },
    getWriter: {
        enumerable: !0
    },
    locked: {
        enumerable: !0
    }
}), o(WritableStream.prototype.abort, "abort"), o(WritableStream.prototype.close, "close"), o(WritableStream.prototype.getWriter, "getWriter"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStream.prototype, Symbol.toStringTag, {
    value: "WritableStream",
    configurable: !0
});
class WritableStreamDefaultWriter {
    constructor(e){
        if ($(e, 1, "WritableStreamDefaultWriter"), _t(e, "First parameter"), vt(e)) throw TypeError("This stream has already been locked for exclusive writing by another writer");
        this._ownerWritableStream = e, e._writer = this;
        let t = e._state;
        if ("writable" === t) !qt(e) && e._backpressure ? Zt(this) : (Zt(this), or(this)), Gt(this);
        else if ("erroring" === t) er(this, e._storedError), Gt(this);
        else if ("closed" === t) Zt(this), or(this), Gt(this), Kt(this);
        else {
            let t = e._storedError;
            er(this, t), Gt(this), Jt(this, t);
        }
    }
    get closed() {
        return Ot(this) ? this._closedPromise : l(Vt("closed"));
    }
    get desiredSize() {
        if (!Ot(this)) throw Vt("desiredSize");
        if (void 0 === this._ownerWritableStream) throw Ut("desiredSize");
        return function(e) {
            let t = e._ownerWritableStream, r = t._state;
            return "errored" === r || "erroring" === r ? null : "closed" === r ? 0 : $t(t._writableStreamController);
        }(this);
    }
    get ready() {
        return Ot(this) ? this._readyPromise : l(Vt("ready"));
    }
    abort(e) {
        return Ot(this) ? void 0 === this._ownerWritableStream ? l(Ut("abort")) : wt(this._ownerWritableStream, e) : l(Vt("abort"));
    }
    close() {
        if (!Ot(this)) return l(Vt("close"));
        let e = this._ownerWritableStream;
        return void 0 === e ? l(Ut("close")) : qt(e) ? l(TypeError("Cannot close an already-closing stream")) : Bt(this);
    }
    releaseLock() {
        if (!Ot(this)) throw Vt("releaseLock");
        void 0 !== this._ownerWritableStream && At(this);
    }
    write(e) {
        return Ot(this) ? void 0 === this._ownerWritableStream ? l(Ut("write to")) : zt(this, e) : l(Vt("write"));
    }
}
function Ot(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_ownerWritableStream") && e instanceof WritableStreamDefaultWriter;
}
function Bt(e) {
    return Rt(e._ownerWritableStream);
}
function jt(e, t) {
    "pending" === e._readyPromiseState ? rr(e, t) : er(e, t);
}
function At(e) {
    let t = e._ownerWritableStream, r = TypeError("Writer was released and can no longer be used to monitor the stream's closedness");
    jt(e, r), "pending" === e._closedPromiseState || Gt(e), Jt(e, r), t._writer = void 0, e._ownerWritableStream = void 0;
}
function zt(e, t) {
    let r = e._ownerWritableStream, o = r._writableStreamController, n = function(e, t) {
        try {
            return e._strategySizeAlgorithm(t);
        } catch (t) {
            return Yt(e, t), 1;
        }
    }(o, t);
    if (r !== e._ownerWritableStream) return l(Ut("write to"));
    let a = r._state;
    if ("errored" === a) return l(r._storedError);
    if (qt(r) || "closed" === a) return l(TypeError("The stream is closing or closed and cannot be written to"));
    if ("erroring" === a) return l(r._storedError);
    let i = u((t, r1)=>{
        r._writeRequests.push({
            _resolve: t,
            _reject: r1
        });
    });
    return function(e, t, r) {
        try {
            ve(e, t, r);
        } catch (t) {
            return void Yt(e, t);
        }
        let o = e._controlledWritableStream;
        qt(o) || "writable" !== o._state || Wt(o, 0 >= $t(e)), Mt(e);
    }(o, t, n), i;
}
Object.defineProperties(WritableStreamDefaultWriter.prototype, {
    abort: {
        enumerable: !0
    },
    close: {
        enumerable: !0
    },
    releaseLock: {
        enumerable: !0
    },
    write: {
        enumerable: !0
    },
    closed: {
        enumerable: !0
    },
    desiredSize: {
        enumerable: !0
    },
    ready: {
        enumerable: !0
    }
}), o(WritableStreamDefaultWriter.prototype.abort, "abort"), o(WritableStreamDefaultWriter.prototype.close, "close"), o(WritableStreamDefaultWriter.prototype.releaseLock, "releaseLock"), o(WritableStreamDefaultWriter.prototype.write, "write"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStreamDefaultWriter.prototype, Symbol.toStringTag, {
    value: "WritableStreamDefaultWriter",
    configurable: !0
});
let Dt = {};
class WritableStreamDefaultController {
    constructor(){
        throw TypeError("Illegal constructor");
    }
    get abortReason() {
        if (!Lt(this)) throw Ht("abortReason");
        return this._abortReason;
    }
    get signal() {
        if (!Lt(this)) throw Ht("signal");
        if (void 0 === this._abortController) throw TypeError("WritableStreamDefaultController.prototype.signal is not supported");
        return this._abortController.signal;
    }
    error(e) {
        if (!Lt(this)) throw Ht("error");
        "writable" === this._controlledWritableStream._state && Qt(this, e);
    }
    [w](e) {
        let t = this._abortAlgorithm(e);
        return It(this), t;
    }
    [R]() {
        we(this);
    }
}
function Lt(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_controlledWritableStream") && e instanceof WritableStreamDefaultController;
}
function Ft(e, t, r, o, n, a, i, l) {
    t._controlledWritableStream = e, e._writableStreamController = t, t._queue = void 0, t._queueTotalSize = void 0, we(t), t._abortReason = void 0, t._abortController = function() {
        if (pt) return new AbortController;
    }(), t._started = !1, t._strategySizeAlgorithm = l, t._strategyHWM = i, t._writeAlgorithm = o, t._closeAlgorithm = n, t._abortAlgorithm = a, Wt(e, 0 >= $t(t)), b(c(r()), ()=>(t._started = !0, Mt(t), null), (r)=>(t._started = !0, Tt(e, r), null));
}
function It(e) {
    e._writeAlgorithm = void 0, e._closeAlgorithm = void 0, e._abortAlgorithm = void 0, e._strategySizeAlgorithm = void 0;
}
function $t(e) {
    return e._strategyHWM - e._queueTotalSize;
}
function Mt(e) {
    let t = e._controlledWritableStream;
    if (!e._started || void 0 !== t._inFlightWriteRequest) return;
    if ("erroring" === t._state) return void Pt(t);
    if (0 === e._queue.length) return;
    let r = e._queue.peek().value;
    r === Dt ? function(e) {
        let t = e._controlledWritableStream;
        t._inFlightCloseRequest = t._closeRequest, t._closeRequest = void 0, ge(e);
        let r = e._closeAlgorithm();
        It(e), b(r, ()=>((function(e) {
                e._inFlightCloseRequest._resolve(void 0), e._inFlightCloseRequest = void 0, "erroring" === e._state && (e._storedError = void 0, void 0 !== e._pendingAbortRequest && (e._pendingAbortRequest._resolve(), e._pendingAbortRequest = void 0)), e._state = "closed";
                let t = e._writer;
                void 0 !== t && Kt(t);
            })(t), null), (e)=>(t._inFlightCloseRequest._reject(e), t._inFlightCloseRequest = void 0, void 0 !== t._pendingAbortRequest && (t._pendingAbortRequest._reject(e), t._pendingAbortRequest = void 0), Tt(t, e), null));
    }(e) : function(e, t) {
        let r = e._controlledWritableStream;
        r._inFlightWriteRequest = r._writeRequests.shift(), b(e._writeAlgorithm(t), ()=>{
            r._inFlightWriteRequest._resolve(void 0), r._inFlightWriteRequest = void 0;
            let t = r._state;
            return ge(e), qt(r) || "writable" !== t || Wt(r, 0 >= $t(e)), Mt(e), null;
        }, (t)=>("writable" === r._state && It(e), r._inFlightWriteRequest._reject(t), r._inFlightWriteRequest = void 0, Tt(r, t), null));
    }(e, r);
}
function Yt(e, t) {
    "writable" === e._controlledWritableStream._state && Qt(e, t);
}
function Qt(e, t) {
    let r = e._controlledWritableStream;
    It(e), Ct(r, t);
}
function Nt(e) {
    return TypeError(`WritableStream.prototype.${e} can only be used on a WritableStream`);
}
function Ht(e) {
    return TypeError(`WritableStreamDefaultController.prototype.${e} can only be used on a WritableStreamDefaultController`);
}
function Vt(e) {
    return TypeError(`WritableStreamDefaultWriter.prototype.${e} can only be used on a WritableStreamDefaultWriter`);
}
function Ut(e) {
    return TypeError("Cannot " + e + " a stream using a released writer");
}
function Gt(e) {
    e._closedPromise = u((t, r)=>{
        e._closedPromise_resolve = t, e._closedPromise_reject = r, e._closedPromiseState = "pending";
    });
}
function Jt(e, t) {
    void 0 !== e._closedPromise_reject && (p(e._closedPromise), e._closedPromise_reject(t), e._closedPromise_resolve = void 0, e._closedPromise_reject = void 0, e._closedPromiseState = "rejected");
}
function Kt(e) {
    void 0 !== e._closedPromise_resolve && (e._closedPromise_resolve(void 0), e._closedPromise_resolve = void 0, e._closedPromise_reject = void 0, e._closedPromiseState = "resolved");
}
function Zt(e) {
    e._readyPromise = u((t, r)=>{
        e._readyPromise_resolve = t, e._readyPromise_reject = r;
    }), e._readyPromiseState = "pending";
}
function er(e, t) {
    Zt(e), rr(e, t);
}
function rr(e, t) {
    void 0 !== e._readyPromise_reject && (p(e._readyPromise), e._readyPromise_reject(t), e._readyPromise_resolve = void 0, e._readyPromise_reject = void 0, e._readyPromiseState = "rejected");
}
function or(e) {
    void 0 !== e._readyPromise_resolve && (e._readyPromise_resolve(void 0), e._readyPromise_resolve = void 0, e._readyPromise_reject = void 0, e._readyPromiseState = "fulfilled");
}
Object.defineProperties(WritableStreamDefaultController.prototype, {
    abortReason: {
        enumerable: !0
    },
    signal: {
        enumerable: !0
    },
    error: {
        enumerable: !0
    }
}), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(WritableStreamDefaultController.prototype, Symbol.toStringTag, {
    value: "WritableStreamDefaultController",
    configurable: !0
});
let nr = "undefined" != typeof globalThis ? globalThis : "undefined" != typeof self ? self : "undefined" != typeof global ? global : void 0, ar = function() {
    let e = null == nr ? void 0 : nr.DOMException;
    return !function(e) {
        if ("function" != typeof e && "object" != typeof e || "DOMException" !== e.name) return !1;
        try {
            return new e, !0;
        } catch (e) {
            return !1;
        }
    }(e) ? void 0 : e;
}() || function() {
    let e = function(e, t) {
        this.message = e || "", this.name = t || "Error", Error.captureStackTrace && Error.captureStackTrace(this, this.constructor);
    };
    return o(e, "DOMException"), e.prototype = Object.create(Error.prototype), Object.defineProperty(e.prototype, "constructor", {
        value: e,
        writable: !0,
        configurable: !0
    }), e;
}();
function ir(t, r, o, n, a, i) {
    let l1 = H(t), s = new WritableStreamDefaultWriter(r);
    t._disturbed = !0;
    let _ = !1, y = c(void 0);
    return u((S, g)=>{
        var R, T;
        let v;
        if (void 0 !== i) {
            if (v = ()=>{
                let e = void 0 !== i.reason ? i.reason : new ar("Aborted", "AbortError"), o = [];
                n || o.push(()=>"writable" === r._state ? wt(r, e) : c(void 0)), a || o.push(()=>"readable" === t._state ? Or(t, e) : c(void 0)), q(()=>Promise.all(o.map((e)=>e())), !0, e);
            }, i.aborted) return void v();
            i.addEventListener("abort", v);
        }
        if (P(t, l1._closedPromise, (e)=>(n ? E(!0, e) : q(()=>wt(r, e), !0, e), null)), P(r, s._closedPromise, (e)=>(a ? E(!0, e) : q(()=>Or(t, e), !0, e), null)), R = l1._closedPromise, T = ()=>(o ? E() : q(()=>(function(e) {
                    let t = e._ownerWritableStream, r = t._state;
                    return qt(t) || "closed" === r ? c(void 0) : "errored" === r ? l(t._storedError) : Bt(e);
                })(s)), null), "closed" === t._state ? T() : b(R, T), qt(r) || "closed" === r._state) {
            let e = TypeError("the destination writable stream closed before all data could be piped to it");
            a ? E(!0, e) : q(()=>Or(t, e), !0, e);
        }
        function C() {
            let e = y;
            return f(y, ()=>e !== y ? C() : void 0);
        }
        function P(e, t, r) {
            "errored" === e._state ? r(e._storedError) : m(t, r);
        }
        function q(e, t, o) {
            function n() {
                return b(e(), ()=>O(t, o), (e)=>O(!0, e)), null;
            }
            _ || (_ = !0, "writable" !== r._state || qt(r) ? n() : b(C(), n));
        }
        function E(e, t) {
            _ || (_ = !0, "writable" !== r._state || qt(r) ? O(e, t) : b(C(), ()=>O(e, t)));
        }
        function O(e, t) {
            return At(s), W(l1), void 0 !== i && i.removeEventListener("abort", v), e ? g(t) : S(void 0), null;
        }
        p(u((t, r)=>{
            !function o(n) {
                n ? t() : f(_ ? c(!0) : f(s._readyPromise, ()=>u((t, r)=>{
                        K(l1, {
                            _chunkSteps: (r)=>{
                                y = f(zt(s, r), void 0, e), t(!1);
                            },
                            _closeSteps: ()=>t(!0),
                            _errorSteps: r
                        });
                    })), o, r);
            }(!1);
        }));
    });
}
class ReadableStreamDefaultController {
    constructor(){
        throw TypeError("Illegal constructor");
    }
    get desiredSize() {
        if (!lr(this)) throw pr("desiredSize");
        return hr(this);
    }
    close() {
        if (!lr(this)) throw pr("close");
        if (!mr(this)) throw TypeError("The stream is not in a state that permits close");
        dr(this);
    }
    enqueue(e) {
        if (!lr(this)) throw pr("enqueue");
        if (!mr(this)) throw TypeError("The stream is not in a state that permits enqueue");
        return fr(this, e);
    }
    error(e) {
        if (!lr(this)) throw pr("error");
        br(this, e);
    }
    [T](e) {
        we(this);
        let t = this._cancelAlgorithm(e);
        return cr(this), t;
    }
    [C](e) {
        let t = this._controlledReadableStream;
        if (this._queue.length > 0) {
            let r = ge(this);
            this._closeRequested && 0 === this._queue.length ? (cr(this), Br(t)) : sr(this), e._chunkSteps(r);
        } else V(t, e), sr(this);
    }
    [P]() {}
}
function lr(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_controlledReadableStream") && e instanceof ReadableStreamDefaultController;
}
function sr(e) {
    if (ur(e)) {
        if (e._pulling) return void (e._pullAgain = !0);
        e._pulling = !0, b(e._pullAlgorithm(), ()=>(e._pulling = !1, e._pullAgain && (e._pullAgain = !1, sr(e)), null), (t)=>(br(e, t), null));
    }
}
function ur(e) {
    let t = e._controlledReadableStream;
    return !!mr(e) && !!e._started && (!!(Wr(t) && G(t) > 0) || hr(e) > 0);
}
function cr(e) {
    e._pullAlgorithm = void 0, e._cancelAlgorithm = void 0, e._strategySizeAlgorithm = void 0;
}
function dr(e) {
    if (!mr(e)) return;
    let t = e._controlledReadableStream;
    e._closeRequested = !0, 0 === e._queue.length && (cr(e), Br(t));
}
function fr(e, t) {
    if (!mr(e)) return;
    let r = e._controlledReadableStream;
    if (Wr(r) && G(r) > 0) U(r, t, !1);
    else {
        let r;
        try {
            r = e._strategySizeAlgorithm(t);
        } catch (t) {
            throw br(e, t), t;
        }
        try {
            ve(e, t, r);
        } catch (t) {
            throw br(e, t), t;
        }
    }
    sr(e);
}
function br(e, t) {
    let r = e._controlledReadableStream;
    "readable" === r._state && (we(e), cr(e), kr(r, t));
}
function hr(e) {
    let t = e._controlledReadableStream._state;
    return "errored" === t ? null : "closed" === t ? 0 : e._strategyHWM - e._queueTotalSize;
}
function mr(e) {
    let t = e._controlledReadableStream._state;
    return !e._closeRequested && "readable" === t;
}
function _r(e, t, r, o, n, a, i) {
    t._controlledReadableStream = e, t._queue = void 0, t._queueTotalSize = void 0, we(t), t._started = !1, t._closeRequested = !1, t._pullAgain = !1, t._pulling = !1, t._strategySizeAlgorithm = i, t._strategyHWM = a, t._pullAlgorithm = o, t._cancelAlgorithm = n, e._readableStreamController = t, b(c(r()), ()=>(t._started = !0, sr(t), null), (e)=>(br(t, e), null));
}
function pr(e) {
    return TypeError(`ReadableStreamDefaultController.prototype.${e} can only be used on a ReadableStreamDefaultController`);
}
function Tr(e, t) {
    L(e, t);
    let r = null == e ? void 0 : e.preventAbort, o = null == e ? void 0 : e.preventCancel, n = null == e ? void 0 : e.preventClose, a = null == e ? void 0 : e.signal;
    return void 0 !== a && function(e, t) {
        if (!function(e) {
            if ("object" != typeof e || null === e) return !1;
            try {
                return "boolean" == typeof e.aborted;
            } catch (e) {
                return !1;
            }
        }(e)) throw TypeError(`${t} is not an AbortSignal.`);
    }(a, `${t} has member 'signal' that`), {
        preventAbort: !!r,
        preventCancel: !!o,
        preventClose: !!n,
        signal: a
    };
}
Object.defineProperties(ReadableStreamDefaultController.prototype, {
    close: {
        enumerable: !0
    },
    enqueue: {
        enumerable: !0
    },
    error: {
        enumerable: !0
    },
    desiredSize: {
        enumerable: !0
    }
}), o(ReadableStreamDefaultController.prototype.close, "close"), o(ReadableStreamDefaultController.prototype.enqueue, "enqueue"), o(ReadableStreamDefaultController.prototype.error, "error"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStreamDefaultController.prototype, Symbol.toStringTag, {
    value: "ReadableStreamDefaultController",
    configurable: !0
});
class ReadableStream {
    constructor(e = {}, t = {}){
        void 0 === e ? e = null : I(e, "First parameter");
        let r = ct(t, "Second parameter"), o = function(e, t) {
            L(e, t);
            let o = null == e ? void 0 : e.autoAllocateChunkSize, n = null == e ? void 0 : e.cancel, a = null == e ? void 0 : e.pull, i = null == e ? void 0 : e.start, l = null == e ? void 0 : e.type;
            return {
                autoAllocateChunkSize: void 0 === o ? void 0 : Q(o, `${t} has member 'autoAllocateChunkSize' that`),
                cancel: void 0 === n ? void 0 : (F(n, `${t} has member 'cancel' that`), (r)=>g(n, e, [
                        r
                    ])),
                pull: void 0 === a ? void 0 : (F(a, `${t} has member 'pull' that`), (r)=>g(a, e, [
                        r
                    ])),
                start: void 0 === i ? void 0 : (F(i, `${t} has member 'start' that`), (r)=>S(i, e, [
                        r
                    ])),
                type: void 0 === l ? void 0 : function(e, t) {
                    if ("bytes" != (e = `${e}`)) throw TypeError(`${t} '${e}' is not a valid enumeration value for ReadableStreamType`);
                    return e;
                }(l, `${t} has member 'type' that`)
            };
        }(e, "First parameter");
        if (qr(this), "bytes" === o.type) {
            if (void 0 !== r.size) throw RangeError("The strategy for a byte stream cannot have a size function");
            !function(e, t, r) {
                let n, a, i;
                let o = Object.create(ReadableByteStreamController.prototype);
                n = void 0 !== t.start ? ()=>t.start(o) : ()=>{}, a = void 0 !== t.pull ? ()=>t.pull(o) : ()=>c(void 0), i = void 0 !== t.cancel ? (e)=>t.cancel(e) : ()=>c(void 0);
                let l = t.autoAllocateChunkSize;
                if (0 === l) throw TypeError("autoAllocateChunkSize must be greater than 0");
                Xe(e, o, n, a, i, r, l);
            }(this, o, st(r, 0));
        } else {
            let e = ut(r);
            !function(e, t, r, o) {
                let a, i;
                let n = Object.create(ReadableStreamDefaultController.prototype);
                a = void 0 !== t.start ? ()=>t.start(n) : ()=>{}, i = void 0 !== t.pull ? ()=>t.pull(n) : ()=>c(void 0), _r(e, n, a, i, void 0 !== t.cancel ? (e)=>t.cancel(e) : ()=>c(void 0), r, o);
            }(this, o, st(r, 1), e);
        }
    }
    get locked() {
        if (!Er(this)) throw jr("locked");
        return Wr(this);
    }
    cancel(e) {
        return Er(this) ? Wr(this) ? l(TypeError("Cannot cancel a stream that already has a reader")) : Or(this, e) : l(jr("cancel"));
    }
    getReader(e) {
        if (!Er(this)) throw jr("getReader");
        return void 0 === function(e, t) {
            L(e, t);
            let r = null == e ? void 0 : e.mode;
            return {
                mode: void 0 === r ? void 0 : function(e, t) {
                    if ("byob" != (e = `${e}`)) throw TypeError(`${t} '${e}' is not a valid enumeration value for ReadableStreamReaderMode`);
                    return e;
                }(r, `${t} has member 'mode' that`)
            };
        }(e, "First parameter").mode ? H(this) : new ReadableStreamBYOBReader(this);
    }
    pipeThrough(e, t = {}) {
        if (!Er(this)) throw jr("pipeThrough");
        $(e, 1, "pipeThrough");
        let r = function(e, t) {
            L(e, t);
            let r = null == e ? void 0 : e.readable;
            M(r, "readable", "ReadableWritablePair"), N(r, `${t} has member 'readable' that`);
            let o = null == e ? void 0 : e.writable;
            return M(o, "writable", "ReadableWritablePair"), _t(o, `${t} has member 'writable' that`), {
                readable: r,
                writable: o
            };
        }(e, "First parameter"), o = Tr(t, "Second parameter");
        if (Wr(this)) throw TypeError("ReadableStream.prototype.pipeThrough cannot be used on a locked ReadableStream");
        if (vt(r.writable)) throw TypeError("ReadableStream.prototype.pipeThrough cannot be used on a locked WritableStream");
        return p(ir(this, r.writable, o.preventClose, o.preventAbort, o.preventCancel, o.signal)), r.readable;
    }
    pipeTo(e, t = {}) {
        let r;
        if (!Er(this)) return l(jr("pipeTo"));
        if (void 0 === e) return l("Parameter 1 is required in 'pipeTo'.");
        if (!gt(e)) return l(TypeError("ReadableStream.prototype.pipeTo's first argument must be a WritableStream"));
        try {
            r = Tr(t, "Second parameter");
        } catch (e) {
            return l(e);
        }
        return Wr(this) ? l(TypeError("ReadableStream.prototype.pipeTo cannot be used on a locked ReadableStream")) : vt(e) ? l(TypeError("ReadableStream.prototype.pipeTo cannot be used on a locked WritableStream")) : ir(this, e, r.preventClose, r.preventAbort, r.preventCancel, r.signal);
    }
    tee() {
        if (!Er(this)) throw jr("tee");
        return ne(Te(this._readableStreamController) ? function(e) {
            let t, r, o, n, a, i = H(e), l = !1, s = !1, d = !1, f = !1, b = !1, h = u((e)=>{
                a = e;
            });
            function _(e) {
                m(e._closedPromise, (t)=>(e !== i || (Qe(o._readableStreamController, t), Qe(n._readableStreamController, t), f && b || a(void 0)), null));
            }
            function p() {
                nt(i) && (W(i), _(i = H(e))), K(i, {
                    _chunkSteps: (t)=>{
                        y(()=>{
                            s = !1, d = !1;
                            let i = t;
                            if (!f && !b) try {
                                i = Se(t);
                            } catch (t) {
                                return Qe(o._readableStreamController, t), Qe(n._readableStreamController, t), void a(Or(e, t));
                            }
                            f || xe(o._readableStreamController, t), b || xe(n._readableStreamController, i), l = !1, s ? g() : d && v();
                        });
                    },
                    _closeSteps: ()=>{
                        l = !1, f || Ye(o._readableStreamController), b || Ye(n._readableStreamController), o._readableStreamController._pendingPullIntos.length > 0 && Ue(o._readableStreamController, 0), n._readableStreamController._pendingPullIntos.length > 0 && Ue(n._readableStreamController, 0), f && b || a(void 0);
                    },
                    _errorSteps: ()=>{
                        l = !1;
                    }
                });
            }
            function S(t, r) {
                J(i) && (W(i), _(i = new ReadableStreamBYOBReader(e)));
                let u = r ? n : o, c = r ? o : n;
                at(i, t, 1, {
                    _chunkSteps: (t)=>{
                        y(()=>{
                            s = !1, d = !1;
                            let o = r ? b : f;
                            if (r ? f : b) o || Ge(u._readableStreamController, t);
                            else {
                                let r;
                                try {
                                    r = Se(t);
                                } catch (t) {
                                    return Qe(u._readableStreamController, t), Qe(c._readableStreamController, t), void a(Or(e, t));
                                }
                                o || Ge(u._readableStreamController, t), xe(c._readableStreamController, r);
                            }
                            l = !1, s ? g() : d && v();
                        });
                    },
                    _closeSteps: (e)=>{
                        l = !1;
                        let t = r ? b : f, o = r ? f : b;
                        t || Ye(u._readableStreamController), o || Ye(c._readableStreamController), void 0 !== e && (t || Ge(u._readableStreamController, e), !o && c._readableStreamController._pendingPullIntos.length > 0 && Ue(c._readableStreamController, 0)), t && o || a(void 0);
                    },
                    _errorSteps: ()=>{
                        l = !1;
                    }
                });
            }
            function g() {
                if (l) return s = !0, c(void 0);
                l = !0;
                let e = He(o._readableStreamController);
                return null === e ? p() : S(e._view, !1), c(void 0);
            }
            function v() {
                if (l) return d = !0, c(void 0);
                l = !0;
                let e = He(n._readableStreamController);
                return null === e ? p() : S(e._view, !0), c(void 0);
            }
            function T() {}
            return o = Pr(T, g, function(o) {
                if (f = !0, t = o, b) {
                    let n = Or(e, ne([
                        t,
                        r
                    ]));
                    a(n);
                }
                return h;
            }), n = Pr(T, v, function(o) {
                if (b = !0, r = o, f) {
                    let n = Or(e, ne([
                        t,
                        r
                    ]));
                    a(n);
                }
                return h;
            }), _(i), [
                o,
                n
            ];
        }(this) : function(e, t) {
            let r = H(e), o, n, a, i, l, s = !1, d = !1, f = !1, b = !1, h = u((e)=>{
                l = e;
            });
            function _() {
                return s ? d = !0 : (s = !0, K(r, {
                    _chunkSteps: (e)=>{
                        y(()=>{
                            d = !1, f || fr(a._readableStreamController, e), b || fr(i._readableStreamController, e), s = !1, d && _();
                        });
                    },
                    _closeSteps: ()=>{
                        s = !1, f || dr(a._readableStreamController), b || dr(i._readableStreamController), f && b || l(void 0);
                    },
                    _errorSteps: ()=>{
                        s = !1;
                    }
                })), c(void 0);
            }
            function g() {}
            return a = Cr(g, _, function(t) {
                if (f = !0, o = t, b) {
                    let r = Or(e, ne([
                        o,
                        n
                    ]));
                    l(r);
                }
                return h;
            }), i = Cr(g, _, function(t) {
                if (b = !0, n = t, f) {
                    let r = Or(e, ne([
                        o,
                        n
                    ]));
                    l(r);
                }
                return h;
            }), m(r._closedPromise, (e)=>(br(a._readableStreamController, e), br(i._readableStreamController, e), f && b || l(void 0), null)), [
                a,
                i
            ];
        }(this));
    }
    values(e) {
        if (!Er(this)) throw jr("values");
        return function(e, t) {
            let o = new he(H(e), t), n = Object.create(me);
            return n._asyncIteratorImpl = o, n;
        }(this, (L(e, "First parameter"), {
            preventCancel: !!(null == e ? void 0 : e.preventCancel)
        }).preventCancel);
    }
    [de](e) {
        return this.values(e);
    }
    static from(e1) {
        var r;
        let o;
        return t(e1) && void 0 !== e1.getReader ? (r = e1.getReader(), o = Cr(e, function() {
            let e;
            try {
                e = r.read();
            } catch (e) {
                return l(e);
            }
            return f(e, (e)=>{
                if (!t(e)) throw TypeError("The promise returned by the reader.read() method must fulfill with an object");
                if (e.done) dr(o._readableStreamController);
                else {
                    let t = e.value;
                    fr(o._readableStreamController, t);
                }
            }, void 0);
        }, function(e) {
            try {
                return c(r.cancel(e));
            } catch (e) {
                return l(e);
            }
        }, 0)) : function(r) {
            let o;
            let n = function fe(e, r = "sync", o) {
                if (void 0 === o) {
                    if ("async" === r) {
                        if (void 0 === (o = ue(e, de))) return function(e) {
                            let r = {
                                next () {
                                    let t;
                                    try {
                                        t = be(e);
                                    } catch (e) {
                                        return l(e);
                                    }
                                    return ce(t);
                                },
                                return (r) {
                                    let o;
                                    try {
                                        let t = ue(e.iterator, "return");
                                        if (void 0 === t) return c({
                                            done: !0,
                                            value: r
                                        });
                                        o = S(t, e.iterator, [
                                            r
                                        ]);
                                    } catch (e) {
                                        return l(e);
                                    }
                                    return t(o) ? ce(o) : l(TypeError("The iterator.return() method must return an object"));
                                }
                            };
                            return {
                                iterator: r,
                                nextMethod: r.next,
                                done: !1
                            };
                        }(fe(e, "sync", ue(e, Symbol.iterator)));
                    } else o = ue(e, Symbol.iterator);
                }
                if (void 0 === o) throw TypeError("The object is not iterable");
                let n = S(o, e, []);
                if (!t(n)) throw TypeError("The iterator method must return an object");
                return {
                    iterator: n,
                    nextMethod: n.next,
                    done: !1
                };
            }(r, "async");
            return o = Cr(e, function() {
                let e;
                try {
                    e = be(n);
                } catch (e) {
                    return l(e);
                }
                return f(c(e), (e)=>{
                    if (!t(e)) throw TypeError("The promise returned by the iterator.next() method must fulfill with an object");
                    if (e.done) dr(o._readableStreamController);
                    else {
                        let t = e.value;
                        fr(o._readableStreamController, t);
                    }
                }, void 0);
            }, function(e) {
                let o;
                let r = n.iterator;
                try {
                    o = ue(r, "return");
                } catch (e) {
                    return l(e);
                }
                return void 0 === o ? c(void 0) : f(g(o, r, [
                    e
                ]), (e)=>{
                    if (!t(e)) throw TypeError("The promise returned by the iterator.return() method must fulfill with an object");
                }, void 0);
            }, 0);
        }(e1);
    }
}
function Cr(e, t, r, o = 1, n = ()=>1) {
    let a = Object.create(ReadableStream.prototype);
    return qr(a), _r(a, Object.create(ReadableStreamDefaultController.prototype), e, t, r, o, n), a;
}
function Pr(e, t, r) {
    let o = Object.create(ReadableStream.prototype);
    return qr(o), Xe(o, Object.create(ReadableByteStreamController.prototype), e, t, r, 0, void 0), o;
}
function qr(e) {
    e._state = "readable", e._reader = void 0, e._storedError = void 0, e._disturbed = !1;
}
function Er(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_readableStreamController") && e instanceof ReadableStream;
}
function Wr(e) {
    return void 0 !== e._reader;
}
function Or(t, r) {
    if (t._disturbed = !0, "closed" === t._state) return c(void 0);
    if ("errored" === t._state) return l(t._storedError);
    Br(t);
    let o = t._reader;
    if (void 0 !== o && nt(o)) {
        let e = o._readIntoRequests;
        o._readIntoRequests = new v, e.forEach((e)=>{
            e._closeSteps(void 0);
        });
    }
    return f(t._readableStreamController[T](r), e, void 0);
}
function Br(e) {
    e._state = "closed";
    let t = e._reader;
    if (void 0 !== t && (A(t), J(t))) {
        let e = t._readRequests;
        t._readRequests = new v, e.forEach((e)=>{
            e._closeSteps();
        });
    }
}
function kr(e, t) {
    e._state = "errored", e._storedError = t;
    let r = e._reader;
    void 0 !== r && (j(r, t), J(r) ? Z(r, t) : it(r, t));
}
function jr(e) {
    return TypeError(`ReadableStream.prototype.${e} can only be used on a ReadableStream`);
}
Object.defineProperties(ReadableStream, {
    from: {
        enumerable: !0
    }
}), Object.defineProperties(ReadableStream.prototype, {
    cancel: {
        enumerable: !0
    },
    getReader: {
        enumerable: !0
    },
    pipeThrough: {
        enumerable: !0
    },
    pipeTo: {
        enumerable: !0
    },
    tee: {
        enumerable: !0
    },
    values: {
        enumerable: !0
    },
    locked: {
        enumerable: !0
    }
}), o(ReadableStream.from, "from"), o(ReadableStream.prototype.cancel, "cancel"), o(ReadableStream.prototype.getReader, "getReader"), o(ReadableStream.prototype.pipeThrough, "pipeThrough"), o(ReadableStream.prototype.pipeTo, "pipeTo"), o(ReadableStream.prototype.tee, "tee"), o(ReadableStream.prototype.values, "values"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(ReadableStream.prototype, Symbol.toStringTag, {
    value: "ReadableStream",
    configurable: !0
}), Object.defineProperty(ReadableStream.prototype, de, {
    value: ReadableStream.prototype.values,
    writable: !0,
    configurable: !0
});
let zr = (e)=>e.byteLength;
o(zr, "size");
let Fr = ()=>1;
o(Fr, "size");
class TransformStream$1 {
    constructor(e = {}, t = {}, r = {}){
        let h;
        void 0 === e && (e = null);
        let o = ct(t, "Second parameter"), n = ct(r, "Third parameter"), a = function(e, t) {
            L(e, t);
            let r = null == e ? void 0 : e.cancel, o = null == e ? void 0 : e.flush, n = null == e ? void 0 : e.readableType, a = null == e ? void 0 : e.start, i = null == e ? void 0 : e.transform, l = null == e ? void 0 : e.writableType;
            return {
                cancel: void 0 === r ? void 0 : (F(r, `${t} has member 'cancel' that`), (r1)=>g(r, e, [
                        r1
                    ])),
                flush: void 0 === o ? void 0 : (F(o, `${t} has member 'flush' that`), (r)=>g(o, e, [
                        r
                    ])),
                readableType: n,
                start: void 0 === a ? void 0 : (F(a, `${t} has member 'start' that`), (r)=>S(a, e, [
                        r
                    ])),
                transform: void 0 === i ? void 0 : (F(i, `${t} has member 'transform' that`), (r, o)=>g(i, e, [
                        r,
                        o
                    ])),
                writableType: l
            };
        }(e, "First parameter");
        if (void 0 !== a.readableType) throw RangeError("Invalid readableType specified");
        if (void 0 !== a.writableType) throw RangeError("Invalid writableType specified");
        let i = st(n, 0), l1 = ut(n), s = st(o, 1), f1 = ut(o);
        !function(e, t, r, o, n, a) {
            function i() {
                return t;
            }
            e._writable = function(e, t, r, o, n = 1, a = ()=>1) {
                let i = Object.create(WritableStream.prototype);
                return St(i), Ft(i, Object.create(WritableStreamDefaultController.prototype), e, t, r, o, n, a), i;
            }(i, function(t) {
                return function(e, t) {
                    let r = e._transformStreamController;
                    return e._backpressure ? f(e._backpressureChangePromise, ()=>{
                        let o = e._writable;
                        if ("erroring" === o._state) throw o._storedError;
                        return Zr(r, t);
                    }, void 0) : Zr(r, t);
                }(e, t);
            }, function() {
                return function(e) {
                    let t = e._transformStreamController;
                    if (void 0 !== t._finishPromise) return t._finishPromise;
                    let r = e._readable;
                    t._finishPromise = u((e, r)=>{
                        t._finishPromise_resolve = e, t._finishPromise_reject = r;
                    });
                    let o = t._flushAlgorithm();
                    return Jr(t), b(o, ()=>("errored" === r._state ? ro(t, r._storedError) : (dr(r._readableStreamController), to(t)), null), (e)=>(br(r._readableStreamController, e), ro(t, e), null)), t._finishPromise;
                }(e);
            }, function(t) {
                return function(e, t) {
                    let r = e._transformStreamController;
                    if (void 0 !== r._finishPromise) return r._finishPromise;
                    let o = e._readable;
                    r._finishPromise = u((e, t)=>{
                        r._finishPromise_resolve = e, r._finishPromise_reject = t;
                    });
                    let n = r._cancelAlgorithm(t);
                    return Jr(r), b(n, ()=>("errored" === o._state ? ro(r, o._storedError) : (br(o._readableStreamController, t), to(r)), null), (e)=>(br(o._readableStreamController, e), ro(r, e), null)), r._finishPromise;
                }(e, t);
            }, r, o), e._readable = Cr(i, function() {
                return Gr(e, !1), e._backpressureChangePromise;
            }, function(t) {
                return function(e, t) {
                    let r = e._transformStreamController;
                    if (void 0 !== r._finishPromise) return r._finishPromise;
                    let o = e._writable;
                    r._finishPromise = u((e, t)=>{
                        r._finishPromise_resolve = e, r._finishPromise_reject = t;
                    });
                    let n = r._cancelAlgorithm(t);
                    return Jr(r), b(n, ()=>("errored" === o._state ? ro(r, o._storedError) : (Yt(o._writableStreamController, t), Ur(e), to(r)), null), (t)=>(Yt(o._writableStreamController, t), Ur(e), ro(r, t), null)), r._finishPromise;
                }(e, t);
            }, n, a), e._backpressure = void 0, e._backpressureChangePromise = void 0, e._backpressureChangePromise_resolve = void 0, Gr(e, !0), e._transformStreamController = void 0;
        }(this, u((e)=>{
            h = e;
        }), s, f1, i, l1), function(e, t) {
            let o, n, a;
            let r = Object.create(TransformStreamDefaultController.prototype);
            o = void 0 !== t.transform ? (e)=>t.transform(e, r) : (e)=>{
                try {
                    return Kr(r, e), c(void 0);
                } catch (e) {
                    return l(e);
                }
            }, n = void 0 !== t.flush ? ()=>t.flush(r) : ()=>c(void 0), a = void 0 !== t.cancel ? (e)=>t.cancel(e) : ()=>c(void 0), r._controlledTransformStream = e, e._transformStreamController = r, r._transformAlgorithm = o, r._flushAlgorithm = n, r._cancelAlgorithm = a, r._finishPromise = void 0, r._finishPromise_resolve = void 0, r._finishPromise_reject = void 0;
        }(this, a), void 0 !== a.start ? h(a.start(this._transformStreamController)) : h(void 0);
    }
    get readable() {
        if (!Nr(this)) throw oo("readable");
        return this._readable;
    }
    get writable() {
        if (!Nr(this)) throw oo("writable");
        return this._writable;
    }
}
function Nr(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_transformStreamController") && e instanceof TransformStream$1;
}
function Hr(e, t) {
    br(e._readable._readableStreamController, t), Vr(e, t);
}
function Vr(e, t) {
    Jr(e._transformStreamController), Yt(e._writable._writableStreamController, t), Ur(e);
}
function Ur(e) {
    e._backpressure && Gr(e, !1);
}
function Gr(e, t) {
    void 0 !== e._backpressureChangePromise && e._backpressureChangePromise_resolve(), e._backpressureChangePromise = u((t)=>{
        e._backpressureChangePromise_resolve = t;
    }), e._backpressure = t;
}
Object.defineProperties(TransformStream$1.prototype, {
    readable: {
        enumerable: !0
    },
    writable: {
        enumerable: !0
    }
}), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(TransformStream$1.prototype, Symbol.toStringTag, {
    value: "TransformStream",
    configurable: !0
});
class TransformStreamDefaultController {
    constructor(){
        throw TypeError("Illegal constructor");
    }
    get desiredSize() {
        if (!Xr(this)) throw eo("desiredSize");
        return hr(this._controlledTransformStream._readable._readableStreamController);
    }
    enqueue(e) {
        if (!Xr(this)) throw eo("enqueue");
        Kr(this, e);
    }
    error(e) {
        if (!Xr(this)) throw eo("error");
        Hr(this._controlledTransformStream, e);
    }
    terminate() {
        if (!Xr(this)) throw eo("terminate");
        !function(e) {
            let t = e._controlledTransformStream;
            dr(t._readable._readableStreamController), Vr(t, TypeError("TransformStream terminated"));
        }(this);
    }
}
function Xr(e) {
    return !!t(e) && !!Object.prototype.hasOwnProperty.call(e, "_controlledTransformStream") && e instanceof TransformStreamDefaultController;
}
function Jr(e) {
    e._transformAlgorithm = void 0, e._flushAlgorithm = void 0, e._cancelAlgorithm = void 0;
}
function Kr(e, t) {
    let r = e._controlledTransformStream, o = r._readable._readableStreamController;
    if (!mr(o)) throw TypeError("Readable side is not in a state that permits enqueue");
    try {
        fr(o, t);
    } catch (e) {
        throw Vr(r, e), r._readable._storedError;
    }
    !ur(o) !== r._backpressure && Gr(r, !0);
}
function Zr(e, t) {
    return f(e._transformAlgorithm(t), void 0, (t)=>{
        throw Hr(e._controlledTransformStream, t), t;
    });
}
function eo(e) {
    return TypeError(`TransformStreamDefaultController.prototype.${e} can only be used on a TransformStreamDefaultController`);
}
function to(e) {
    void 0 !== e._finishPromise_resolve && (e._finishPromise_resolve(), e._finishPromise_resolve = void 0, e._finishPromise_reject = void 0);
}
function ro(e, t) {
    void 0 !== e._finishPromise_reject && (p(e._finishPromise), e._finishPromise_reject(t), e._finishPromise_resolve = void 0, e._finishPromise_reject = void 0);
}
function oo(e) {
    return TypeError(`TransformStream.prototype.${e} can only be used on a TransformStream`);
}
Object.defineProperties(TransformStreamDefaultController.prototype, {
    enqueue: {
        enumerable: !0
    },
    error: {
        enumerable: !0
    },
    terminate: {
        enumerable: !0
    },
    desiredSize: {
        enumerable: !0
    }
}), o(TransformStreamDefaultController.prototype.enqueue, "enqueue"), o(TransformStreamDefaultController.prototype.error, "error"), o(TransformStreamDefaultController.prototype.terminate, "terminate"), "symbol" == typeof Symbol.toStringTag && Object.defineProperty(TransformStreamDefaultController.prototype, Symbol.toStringTag, {
    value: "TransformStreamDefaultController",
    configurable: !0
});

let decDecoder = Symbol("decDecoder"), decTransform = Symbol("decTransform");
class TextDecodeTransformer {
    constructor(decoder){
        this.decoder_ = decoder;
    }
    transform(chunk, controller) {
        if (!(chunk instanceof ArrayBuffer || ArrayBuffer.isView(chunk))) throw TypeError("Input data must be a BufferSource");
        let text = this.decoder_.decode(chunk, {
            stream: !0
        });
        0 !== text.length && controller.enqueue(text);
    }
    flush(controller) {
        let text = this.decoder_.decode();
        0 !== text.length && controller.enqueue(text);
    }
}
class TextDecoderStream {
    constructor(label, options){
        this[decDecoder] = new TextDecoder(label, options), this[decTransform] = new TransformStream(new TextDecodeTransformer(this[decDecoder]));
    }
    get encoding() {
        return this[decDecoder].encoding;
    }
    get fatal() {
        return this[decDecoder].fatal;
    }
    get ignoreBOM() {
        return this[decDecoder].ignoreBOM;
    }
    get readable() {
        return this[decTransform].readable;
    }
    get writable() {
        return this[decTransform].writable;
    }
}
let encEncoder = Symbol("encEncoder"), encTransform = Symbol("encTransform");
class TextEncodeTransformer {
    constructor(encoder){
        this.encoder_ = encoder, this.partial_ = void 0;
    }
    transform(chunk, controller) {
        let stringChunk = String(chunk);
        void 0 !== this.partial_ && (stringChunk = this.partial_ + stringChunk, this.partial_ = void 0);
        let lastCharIndex = stringChunk.length - 1, lastCodeUnit = stringChunk.charCodeAt(lastCharIndex);
        lastCodeUnit >= 0xD800 && lastCodeUnit < 0xDC00 && (this.partial_ = String.fromCharCode(lastCodeUnit), stringChunk = stringChunk.substring(0, lastCharIndex));
        let bytes = this.encoder_.encode(stringChunk);
        0 !== bytes.length && controller.enqueue(bytes);
    }
    flush(controller) {
        this.partial_ && (controller.enqueue(this.encoder_.encode(this.partial_)), this.partial_ = void 0);
    }
}
class TextEncoderStream {
    constructor(){
        this[encEncoder] = new TextEncoder(), this[encTransform] = new TransformStream(new TextEncodeTransformer(this[encEncoder]));
    }
    get encoding() {
        return this[encEncoder].encoding;
    }
    get readable() {
        return this[encTransform].readable;
    }
    get writable() {
        return this[encTransform].writable;
    }
}

function init$3(global) {
    Object.defineProperties(global, {
        ReadableStream: {
            value: ReadableStream
        },
        WritableStream: {
            value: WritableStream
        },
        TransformStream: {
            value: TransformStream$1
        },
        TextDecoderStream: {
            value: TextDecoderStream
        },
        TextEncoderStream: {
            value: TextEncoderStream
        }
    });
}

class Console {
    #target;
    constructor(target = ""){
        this.#target = target;
    }
    log(...args) {
        this.#print("info", args);
    }
    warn(...args) {
        this.#print("warn", args);
    }
    error(...args) {
        this.#print("warn", args);
    }
    #print(level, args) {
        let formatted = args.map((m)=>format(m)).join(", ");
        print(`[${level}] ${formatted}`);
    }
}
function init$2(global) {
    Object.defineProperties(global, {
        Console: {
            value: Console
        },
        console: {
            value: new Console()
        }
    });
}
function format(value, embed = !1) {
    if (Array.isArray(value)) return `[${value.map((m)=>format(m, !0)).join(", ")}]`;
    switch(typeof value){
        case "string":
            return embed ? `"${value}"` : value;
        case "number":
        case "undefined":
            return `${value}`;
        case "function":
            return `<Function ${value.name}>`;
        case "object":
            return function(value) {
                let output = "{", count = 0;
                for(let key in value)count++ > 0 && (output += ","), output += ` ${format(key, !0)}: ${format(value[key], !0)}`;
                return output + " }";
            }(value);
    }
}

async function init$1(global) {
    let { TextDecoder, TextEncoder, set_interval, set_timeout, clear_interval, clear_timeout } = await import('@klaver/base');
    Object.assign(global, {
        TextDecoder,
        TextEncoder,
        setInterval: set_interval,
        setTimeout: set_timeout,
        clearInterval: clear_interval,
        clearTimeout: clear_timeout
    });
}

async function init(global) {
    await init$1(global), init$2(global), init$3(global), await init$4(global);
}
await init(globalThis);
