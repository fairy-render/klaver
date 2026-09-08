

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

interface RsaHashedKeyAlgorithm extends KeyAlgorithm {
  modulusLength: number;
  publicExponent: Uint8Array;
  hash: KeyAlgorithm;
}

interface EcKeyAlgorithm extends KeyAlgorithm {
  namedCurve: "P-256" | "P-384";
}

interface CryptoKey {
  readonly type: KeyType;
  readonly extractable: boolean;
  readonly algorithm:
  | AesKeyAlgorithm
  | HmacKeyAlgorithm
  | RsaHashedKeyAlgorithm
  | EcKeyAlgorithm;
  readonly usages: KeyUsage[];
}

declare var CryptoKey: {
  prototype: CryptoKey;
};

interface CryptoKeyPair {
  publicKey: CryptoKey;
  privateKey: CryptoKey;
}

interface JsonWebKey {
  kty: string;
  k?: string;
  alg?: string;
  ext?: boolean;
  key_ops?: string[];
  // RSA
  n?: string;
  e?: string;
  d?: string;
  p?: string;
  q?: string;
  dp?: string;
  dq?: string;
  qi?: string;
  // EC
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

interface RsaHashedKeyGenParams {
  name: "RSASSA-PKCS1-v1_5" | "RSA-OAEP";
  modulusLength: number;
  /** Big-endian bytes, e.g. `new Uint8Array([1, 0, 1])` for 65537. */
  publicExponent: Uint8Array;
  hash: AlgorithmIdentifier;
}

interface RsaHashedImportParams {
  name: "RSASSA-PKCS1-v1_5" | "RSA-OAEP";
  hash: AlgorithmIdentifier;
}

interface RsaOaepParams {
  name: "RSA-OAEP";
  label?: BufferSource;
}

interface EcKeyGenParams {
  name: "ECDSA" | "ECDH";
  namedCurve: "P-256" | "P-384";
}

interface EcKeyImportParams {
  name: "ECDSA" | "ECDH";
  namedCurve: "P-256" | "P-384";
}

interface EcdsaParams {
  name: "ECDSA";
  hash: AlgorithmIdentifier;
}

interface EcdhKeyDeriveParams {
  name: "ECDH";
  public: CryptoKey;
}

declare interface SubtleCrypto {
  digest(
    algo: "SHA-1" | "SHA-256" | "SHA-384" | "SHA-512",
    input: BufferSource,
  ): Promise<ArrayBuffer>;

  encrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;
  decrypt(
    algorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;

  sign(
    algorithm: AlgorithmIdentifier | EcdsaParams,
    key: CryptoKey,
    data: BufferSource,
  ): Promise<ArrayBuffer>;
  verify(
    algorithm: AlgorithmIdentifier | EcdsaParams,
    key: CryptoKey,
    signature: BufferSource,
    data: BufferSource,
  ): Promise<boolean>;

  generateKey(
    algorithm: AesKeyGenParams | HmacKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  generateKey(
    algorithm: RsaHashedKeyGenParams | EcKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKeyPair>;
  importKey(
    format: KeyFormat,
    keyData: BufferSource | JsonWebKey,
    algorithm:
    | "AES-GCM"
    | "AES-CBC"
    | "AES-CTR"
    | HmacImportParams
    | RsaHashedImportParams
    | EcKeyImportParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
  exportKey(format: KeyFormat, key: CryptoKey): Promise<ArrayBuffer | JsonWebKey>;

  deriveBits(
    algorithm: EcdhKeyDeriveParams,
    baseKey: CryptoKey,
    length?: number | null,
  ): Promise<ArrayBuffer>;
  deriveKey(
    algorithm: EcdhKeyDeriveParams,
    baseKey: CryptoKey,
    derivedKeyAlgorithm: AesKeyGenParams | HmacKeyGenParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;

  wrapKey(
    format: KeyFormat,
    key: CryptoKey,
    wrappingKey: CryptoKey,
    wrapAlgorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
  ): Promise<ArrayBuffer>;
  unwrapKey(
    format: KeyFormat,
    wrappedKey: BufferSource,
    unwrappingKey: CryptoKey,
    unwrapAlgorithm: AesGcmParams | AesCbcParams | AesCtrParams | RsaOaepParams,
    unwrappedKeyAlgorithm:
    | "AES-GCM"
    | "AES-CBC"
    | "AES-CTR"
    | HmacImportParams
    | RsaHashedImportParams
    | EcKeyImportParams,
    extractable: boolean,
    keyUsages: KeyUsage[],
  ): Promise<CryptoKey>;
}

declare const crypto: Crypto;
