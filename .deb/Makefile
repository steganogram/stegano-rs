ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
deb:
	docker pull 5422m4n/rust-deb-builder
	docker run --rm -v ${ROOT_DIR}/..:/mnt -w /mnt \
		5422m4n/rust-deb-builder \
		cargo deb -p stegano-cli --target=x86_64-unknown-linux-musl