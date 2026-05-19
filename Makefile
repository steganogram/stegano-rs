# RPM build Makefile

all: build

# Set up depdencies
setup:
	# Check if cargo-generate-rpm is installed
	@echo "Checking for cargo-generate-rpm..."
	command -v cargo-generate-rpm >/dev/null 2>&1 || cargo install cargo-generate-rpm

# Build
build-rpm: setup
	# Build the stegano binary
	@echo "Building the stegano binary..."
	cargo build --release

	# Strip the binary
	@echo "Stripping the binary..."
	strip -s target/release/stegano

	# Generate the rpm
	@echo "Generating the RPM package..."
	cargo generate-rpm -p crates/stegano-cli

# Install
install:
	# Install the generated RPM package
	@echo "Installing the RPM package..."
	sudo dnf install -y target/generate-rpm/stegano*.rpm

# Clean
clean:
	# Clean the build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
