

declare interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer): void;
}

declare interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

declare const crypto: Crypto;