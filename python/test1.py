from ctypes import cdll

lib = cdll.LoadLibrary("rust/target/debug/libspace.so")

r = lib.add_numbers(3, 2)

print("FFI working: %s" % str(r))

print("Running sample");

lib.execute();

