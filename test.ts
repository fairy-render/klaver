import * as fs from "@klaver/fs";
import { Client, Request } from "@klaver/http";
import { Image } from "@klaver/image";

setTimeout(() => {
  console.log("rappper");
}, 3000);

// const stream = await fs.readDir(await fs.resolve("."));

// for await (const entry of stream) {
// 	console.log(entry.path, entry.type);
// }

// const file = await fs.open("test.ts");

// const lines = await file.readLines();

// for await (const line of lines) {
// 	console.log("line", line);
// }

const now = performance.now();

const client = new Client();

const resp = await client.send(
  new Request(
    "https://dfstudio-d420.kxcdn.com/wordpress/wp-content/uploads/2019/06/digital_camera_photo-1080x675.jpg"
  )
);

const img = new Image(await resp.arrayBuffer());

console.log(img.width, img.height);

// const resizedImage = await img.resize({
// 	width: 200,
// 	height: 200,
// });

// await img.save("image.webp");
// await img.save("image.jpg");

console.log((await img.arrayBuffer("png")).byteLength);

console.log("Took", (performance.now() - now) / 1000);
console.log("origin", performance.timeOrigin);
