import type { Algo, Buffer } from "@klaver/crypto";
import { writeProps } from "./util.js";

export default async function init(global: Record<string, unknown>) {
	const { getRandomValues, randomUUID, Digest } = await import(
		"@klaver/crypto"
	);

	const crypto = {};

	const subtle = {};

	writeProps(subtle, {
		digest: async function digest(algo: Algo, data: Buffer) {
			const hasher = new Digest(algo);
			hasher.update(data);
			return hasher.digest();
		},
	});

	writeProps(crypto, { getRandomValues, randomUUID, subtle });
	writeProps(global, { crypto });
}
