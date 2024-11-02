import ts from "rollup-plugin-ts";
import resolve from "@rollup/plugin-node-resolve";

export default {
	input: "srcjs/index.ts",
	output: {
		file: "src/compat.js",
		format: "es",
	},
	external: ["@klaver/http", "@klaver/encoding"],
	plugins: [
		resolve(),
		ts({
			// transpiler: "swc",
			/* Plugin options */
			hook: {
				// outputPath(path, kind) {
				// 	console.log(path, kind);
				// 	if (kind === "declaration") {
				// 		console.log(path);
				// 		return "./";
				// 	}
				// },
			},
			swcConfig: {
				minify: false,
				jsc: {
					minify: {
						compress: true,
						format: {
							comments: false,
						},
					},
				},
			},
			transpileOnly: true,
		}),
	],
};
