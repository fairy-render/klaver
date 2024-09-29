const modules = ["crypto", "http", "os", "encoding", "base", "fs", "image"];
try {
	await Deno.mkdir("@types");
} catch {}

const encoder = new TextEncoder();

for (const mod of modules) {
	const file = `klaver-${mod}/module.d.ts`;
	const oPath = `@types/@klaver/${mod}`;

	const content = await Deno.readFile(file);

	try {
		await Deno.mkdir(oPath);
	} catch {}

	await Deno.writeFile(
		`${oPath}/package.json`,
		encoder.encode(
			JSON.stringify(
				{
					name: `@klaver/${mod}`,
					types: "index.d.ts",
				},
				null,
				2,
			),
		),
	);

	await Deno.writeFile(`${oPath}/index.d.ts`, content);
}

await Deno.copyFile("klaver/globals.d.ts", "@types/core.d.ts");
await Deno.copyFile("klaver-compat/klaver.d.ts", "@types/klaver.d.ts");
