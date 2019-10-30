if [[ $CLIPPY ]]; then
  cargo clippy --all-targets --all-features -- -D warnings
fi