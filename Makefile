all: linux windows macos

linux:
	cargo build --release

windows:
	rustup target add x86_64-pc-windows-gnu
	sudo apt install -y mingw-w64
	cargo build --release --target x86_64-pc-windows-gnu

macos:
	docker run --rm -v "$$(pwd)":/project -w /project messense/macos-cross-toolchains \
	bash -c "rustup target add x86_64-apple-darwin aarch64-apple-darwin && \
	cargo build --release --target x86_64-apple-darwin && \
	cargo build --release --target aarch64-apple-darwin"