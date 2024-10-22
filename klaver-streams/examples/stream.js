import { ReadableStream } from "@klaver/streams";

const count = 0;
const stream = new ReadableStream(
	{
		async pull(ctrl) {
			await new Promise((res) => Core.timers.createTimer(res, 100, false));
			ctrl.enqueue(`Hello, World ${count}`);

			// if (++count > 10) ctrl.close();
		},

		cancel(reason) {
			print("Reason", reason);
		},
	},
	{
		highWaterMark: 3,
	},
);

let n = 0;
for await (const chunk of stream) {
	print(chunk);
	if (n++ === 4) {
		print("Just because");
		break;
	}
}

await new Promise((res) => Core.timers.createTimer(res, 300, false));
