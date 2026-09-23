
import ctypes
from pathlib import Path

def printc(s: str) -> None:
  lib = ctypes.CDLL(str(Path(__file__).resolve().parent / "libbench.so"))
  
  lib.print.argtypes = (ctypes.c_void_p, ctypes.c_size_t)
  lib.print.restype = ctypes.c_void_p
  data = s.encode("utf-8")
  
  lib.print(data, len(data))
  lib.print(data, len(data))

printc("Hello from RTS 222!")