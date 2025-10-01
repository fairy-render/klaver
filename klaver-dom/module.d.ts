export class Document {
	constructor();

	static parse(html: string): Document;

	readonly body: Element | undefined;

	createElement(tag: string): Element;

	querySelector(selector: string): Element;
	querySelectorAll(select: string): NodeList;
}

export interface Element {
	getAttribute(name: string): string | undefined;

	readonly innerHTML: string;
	readonly innerText: string;

	querySelector(selector: string): Element;
	querySelectorAll(select: string): NodeList;
}

export interface NodeList {
	item(index: number): Element | undefined;
	entries(): IterableIterator<[index: number, element: Element]>;
	values(): IterableIterator<Element>;
}

export interface ClassList {
	readonly length: number;
	readonly value: string;
	item(index: number): string;
	add(...className: string[]): void;
	remove(...className: string[]): void;
	toggle(className: string): void;

	//   entries(): Iterator<[index: number, element: Element]>;
	values(): IterableIterator<string>;
}
