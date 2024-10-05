import * as fs from "@klaver/fs";

const file = await fs.open("./test.ts");

const buffer = new ArrayBuffer(6);

let len = await file.read(buffer);

console.log(new TextDecoder().decode(buffer.slice(0, len)));

len = await file.read(buffer);

console.log(new TextDecoder().decode(buffer.slice(0, len)));

const nfile = await fs.open("Rapper", "wc");

await nfile.write(new TextEncoder().encode("Hello, World"));
