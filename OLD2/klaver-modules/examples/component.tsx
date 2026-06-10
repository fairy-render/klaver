/** @jsxImportSource @klaver/template */

export function Test() {
	return <div>Hello</div>;
}

function inject() {}

class Rapper {}

@inject
class Test {
	constructor(test: Rapper, name: string, age: number) {
		console.log(test);
	}
}
