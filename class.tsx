function test() {
	return (target, ctx: ClassDecoratorContext) => {
		console.log(target);
		ctx.addInitializer(function () {
			console.log(this);
		});
	};
}

@test()
class Test {
	constructor(test: string) {}

	render() {
		// return <div></div>;
	}
}

console.log("hhelo", new Test(""), typeof Reflect);
