export function toUint8Array(value: number[] | Uint8Array) {
  return value instanceof Uint8Array ? value : Uint8Array.from(value)
}
