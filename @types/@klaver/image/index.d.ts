export type ImageFormat = "png" | "jpeg" | "webp" | "ppm";

export type ImageFilter =
	| "nearest"
	| "triangle"
	| "catmullrom"
	| "gaussian"
	| "lanczos3";

export interface ResizeOptions {
	width: number;
	height: number;
	exact?: boolean;
	filter?: ImageFilter;
}

export class Image {
	readonly width: number;
	readonly height: number;

	constructor(bytes: ArrayBuffer);

	static open(path: string, kind?: ImageFormat): Promise<Image>;

	arrayBuffer(format: ImageFormat): Promise<ArrayBuffer>;
	resize(opts: ResizeOptions): Promise<Image>;
	blur(sigma: number): Promise<Image>;
	gray(): Promise<Image>;
	crop(x: number, y: number, width: number, height: number): Promise<Image>;

	save(path: string, kind?: ImageFormat): Promise<void>;
}
