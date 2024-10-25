// const count = 0;
// const stream = new ReadableStream(
// 	{
// 		async pull(ctrl) {
// 			await new Promise((res) => Core.timers.createTimer(res, 100, false));
// 			ctrl.enqueue(`Hello, World ${count}`);

// 			// if (++count > 10) ctrl.close();
// 		},

// 		cancel(reason) {
// 			print("Reason", reason);
// 		},
// 	},
// 	{
// 		highWaterMark: 3,
// 	},
// );

// let n = 0;
// for await (const chunk of stream) {
// 	print(chunk);
// 	if (n++ === 4) {
// 		print("Just because");
// 		break;
// 	}
// }

// const s = ReadableStream.from([1, 2, 3, 4]);

// for await (const n of s) {
// 	print(n);
// }

// const v = ReadableStream.from(
// 	(function* () {
// 		for (const i of [5, 6, 7]) {
// 			yield i;
// 		}
// 	})(),
// );

// for await (const n of v) {
// 	print(n);
// }

// const map = new Map();
// map.set("Hello", "World");

// const v2 = ReadableStream.from(map);

// const reader = v2.getReader();

// reader.closed.then(() => {
// 	print("closed");
// });

// while (true) {
// 	const data = await reader.read();

// 	if (data.done) {
// 		break;
// 	}

// 	print(data.value);
// }

const hashBuffer = await crypto.subtle.digest(
  "sha1",
  new TextEncoder().encode("Hello, World!")
);

const hashArray = Array.from(new Uint8Array(hashBuffer)); // convert buffer to byte array
const hashHex = hashArray.map((b) => b.toString(16).padStart(2, "0")).join(""); // convert bytes to hex string

console.log(hashHex);

const url = new URLSearchParams("hello=world");

console.log(url.toString());
