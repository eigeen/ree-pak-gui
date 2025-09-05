/**
 * Computes the SHA-256 hash of the input array buffer.
 * @param array
 * @returns Hex string of SHA-256 hash of the input array buffer.
 */
export async function sha256Hex(array: ArrayBuffer | Uint8Array): Promise<string> {
  const hash = await crypto.subtle.digest('SHA-256', array)
  const sha256Hex = Array.from(new Uint8Array(hash), (byte) =>
    byte.toString(16).padStart(2, '0')
  ).join('')
  return sha256Hex
}
