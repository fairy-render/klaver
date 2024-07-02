import { writeProp } from "./util.js";

export default function init(global: Record<string, unknown>) {
	writeProp(global, "process", {
		env: {},
	});
}
