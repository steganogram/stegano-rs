if [[ $COVERAGE ]]; then
    cargo tarpaulin --out Xml
    bash <(curl -s https://codecov.io/bash)
fi