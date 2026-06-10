declare class TextEncoder {
    constructor(label?: string);

    readonly encoding: string;
    encode(input: string): Uint8Array;
}

declare class TextDecoder {
    constructor(label?: string);

    readonly encoding: string;
    decode(input: ArrayBuffer): string;
}

declare function atob(input: string): string;
declare function btoa(input: string): string;
