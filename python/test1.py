from ctypes import cdll

lib = cdll.LoadLibrary("target/release/libspace.so")

r = lib.process(3)

print("done! %s" % str(r))
