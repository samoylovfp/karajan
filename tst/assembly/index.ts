// The entry file of your WebAssembly module.

export function processUpdate(p: string): string {
  if (p.length == 0) {
    throw new Error("Hello");
  }
  return `Hi ${p}: 123`
}
