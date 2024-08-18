import { ReadableStream } from "@klaver/streams";

function delay(n: number) {
	return new Promise((res) => setTimeout(res, n));
}

let times = 0;

const stream = new ReadableStream({
	async pull(controller) {
		if (times > 20) {
			controller.close();
			return;
		}
		await delay(100);
		times++;
		controller.enqueue("Rapper " + times);
	},
});

// const reader = stream.getReader();

let count = 0;

for await (const value of stream) {
	if (count++ > 5) {
		break;
	}
	console.log("value", value);
}

// while (true) {
// 	const { value, done } = await reader.read();

// 	if (done) break;

// 	if (count++ == 5) {
// 		await reader.cancel("");
// 	}

// 	console.log("value", value);
// }

console.log("done");
