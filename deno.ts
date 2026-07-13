import { render } from './solid-ssr/dist/server/entry-server.mjs' with { swc: "deno" };

console.time('render')
const out = await render("");
console.timeEnd('render')
console.log('HTML:', out.html);
console.log('Hydration Script:', out.hydration);