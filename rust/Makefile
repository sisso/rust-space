clean:
	@echo "nothing"

build-rust:
	cargo build

copy-generated:
	@echo "copying generated files"
	rsync -v space-flap/generated_dotnet/*.cs ../unity-space/Assets/src/ffi_domain_2
	cp -v target/debug/libspace_flap.so ../unity-space/Assets/Plugins/libffi_domain_2_native.so
#   rsync -vr wasm/css/ wasm/pkg/css
#   rsync -vr wasm/js/ wasm/pkg/js

all: build-rust copy-generated
	@echo "done"
