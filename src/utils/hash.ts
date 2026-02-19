/**
 * Computes the SHA-256 hash of the input array buffer.
 * @param array
 * @returns Hex string of SHA-256 hash of the input array buffer.
 */
export async function sha256Hex(array: ArrayBuffer | Uint8Array): Promise<string> {
  const data =
    array instanceof ArrayBuffer
      ? array
      : (() => {
          const copy = new Uint8Array(array.byteLength)
          copy.set(array)
          return copy.buffer
        })()
  const hash = await crypto.subtle.digest('SHA-256', data)
  const sha256Hex = Array.from(new Uint8Array(hash), (byte) =>
    byte.toString(16).padStart(2, '0')
  ).join('')
  return sha256Hex
}
