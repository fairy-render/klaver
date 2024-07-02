import {
	ReadableStream,
	WritableStream,
	TransformStream,
} from "web-streams-polyfill";
import {
	TextEncoderStream,
	TextDecoderStream,
} from "@stardazed/streams-text-encoding";

export {
	ReadableStream,
	WritableStream,
	TransformStream,
} from "web-streams-polyfill";
export {
	TextEncoderStream,
	TextDecoderStream,
} from "@stardazed/streams-text-encoding";

export default function init(global: Record<string, unknown>) {
	Object.defineProperties(global, {
		ReadableStream: { value: ReadableStream },
		WritableStream: { value: WritableStream },
		TransformStream: { value: TransformStream },
		TextDecoderStream: { value: TextDecoderStream },
		TextEncoderStream: { value: TextEncoderStream },
	});
}
