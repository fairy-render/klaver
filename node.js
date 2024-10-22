async function main() {
	const stream = new ReadableStream({
		pull(ctrl) {
			ctrl.enqueue("Hello, World");
		},
	});

	console.log(stream.locked);

	const reader = stream.getReader();

	console.log(stream.locked);

	for await (const chunk of stream) {
	}

	console.log(stream.locked);
}

main();
