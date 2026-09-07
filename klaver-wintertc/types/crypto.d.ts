

declare interface Crypto {
  readonly subtle: SubtleCrypto;
  randomUUID(): string;
  getRandomValues(buffer: Buffer): void;
}

type KeyUsage =
  | "encrypt"
  | "decrypt"
  | "sign"
  | "verify"
  | "deriveKey"
  | "deriveBits"
  | "wrapKey"
  | "unwrapKey";

type KeyType = "secret" | "private" | "public";
type KeyFormat = "raw" | "pkcs8" | "spki" | "jwk";

type AlgorithmIdentifier = string | { name: string;[key: string]: unknown };
type BufferSource = ArrayBuffer | ArrayBufferView;

interface KeyAlgorithm {
  name: string;
}

interface AesKeyAlgorithm extends KeyAlgorithm {
  length: number;
}

interface HmacKeyAlgorithm extends KeyAlgorithm {
  hash: KeyAlgorithm;
  length: number;
}

interface CryptoKey {
  readonly type: KeyType;
  readonly extractable: boolean;
  readonly algorithm: AesKeyAlgorithm | HmacKeyAlgorithm;
  readonly usages: KeyUsage[];
}

declare var CryptoKey: {
  prototype: CryptoKey;
};

interface JsonWebKey {
  kty: string;
  k?: string;
  alg?: string;
  ext?: boolean;
  key_ops?: string[];
  // Reserved for RSA/EC keys, not yet supported by importKey/exportKey:
  n?: string;
  e?: string;
  d?: string;
  p?: string;
  q?: string;
  dp?: string;
  dq?: string;
  qi?: string;
  crv?: string;
  x?: string;
  y?: string;
}

interface AesKeyGenParams {
  name: "AES-GCM" | "AES-CBC" | "AES-CTR";
  length: 128 | 192 | 256;
}

interface HmacKeyGenParams {
  name: "HMAC";
  hash: AlgorithmIdentifier;
  length?: number;
}

interface HmacImportParams {
  name: "HMAC";
  hash: AlgorithmIdentifier;
}

interface AesGcmParams {
  name: "AES-GCM";
  iv: BufferSource;
  additionalData?: BufferSource;
  /** Bits. Only the default, 128, is currently supported. */
  tagLength?: number;
}

interface AesCbcParams {
  name: "AES-CBC";
  iv: BufferSource;
}

interface AesCtrParams {
  name: "AES-CTR";
  counter: BufferSource;
  /** Bits (1-128) of `counter` that actually form the wrapping counter. */
  length: number;
}

declare interface SubtleCrypto {
  digest(
    algo: "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512",
    input: BufferSource,
  ): Promise<ArrayBuffer>;

  encrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;
  decrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  sign(algorithm: AlgorithmIdentifier, key: CryptoKey, data: BufferSource): Promise<ArrayBuffer>;
  verify(
    algorithm: AlgorithmIdentifier,
    key: CryptoKey,
    signature: BufferSource,
    data: BufferSource,
  ): Promise<boolean>;

  generateKey(
    algorithm: AesKeyGenParams | HmacKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  importKey(
    format: KeyFormat,
    keyData: BufferSource | JsonWebKey,
    algorithm: "AES-GCM" | "AES-CBC" | "AES-CTR" | HmacImportParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  exportKey(format: KeyFormat, key: CryptoKey): Promise<ArrayBuffer | JsonWebKey>;
}

declare const crypto: Crypto;
