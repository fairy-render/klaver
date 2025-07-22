

export interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer);
}

export interface SubtleCrypto {
  digest(algo: "SHA-1" | "SHA-256", input: Buffer): ArrayBuffer;
}

export const crypto: Crypto;