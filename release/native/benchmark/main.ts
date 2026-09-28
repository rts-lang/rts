import { dlopen, FFIType, ptr } from "bun:ffi";
import { join } from "path";

function printc(s: string): void {
  const lib = dlopen(join(import.meta.dir, "libbench.so"), {
    print: { args: [FFIType.ptr, FFIType.u64], returns: FFIType.ptr },
  });
  const data: Uint8Array = new TextEncoder().encode(s);

  lib.symbols.print(ptr(data), data.length);
  lib.symbols.print(ptr(data), data.length);
}

printc("Hello from RTS 222!");