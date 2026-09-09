export {};
//

function encryptMessage(
  message: string,
  publicKey: CryptoKey,
): Promise<ArrayBuffer> {
  const encoder = new TextEncoder();
  const data = encoder.encode(message);
  return crypto.subtle.encrypt({ name: "RSA-OAEP" }, publicKey, data);
}

function decryptMessage(
  encryptedData: ArrayBuffer,
  privateKey: CryptoKey,
): Promise<string> {
  return crypto.subtle
    .decrypt({ name: "RSA-OAEP" }, privateKey, encryptedData)
    .then((decryptedData) => {
      const decoder = new TextDecoder();
      return decoder.decode(decryptedData);
    });
}

try {
  const key = await crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 2048,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["encrypt", "decrypt"],
  );

  const encryptedData = await encryptMessage("Hello, World!", key.publicKey);
  console.log("Encrypted Data", new Uint8Array(encryptedData));

  const decryptedMessage = await decryptMessage(encryptedData, key.privateKey);
  console.log("Decrypted Message", decryptedMessage);

  console.log("key", await crypto.subtle.exportKey("jwk", key.publicKey));
} catch (e) {
  console.log(e.toString());
}
// console.log(key);
