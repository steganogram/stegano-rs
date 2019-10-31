if [[ $COVERAGE ]]; then
  set -e
  wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
  tar xzf master.tar.gz
  cd kcov-master
  mkdir build
  cd build
  cmake ..
  make
  sudo make install
  cd ../..
  rm -rf kcov-master
  for file in target/debug/*[^\.d]; do
    mkdir -p "target/cov/$(basename $file)"
    kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/$(basename $file)" "$file"
  done
  bash <(curl -s https://codecov.io/bash) &&
  echo "Uploaded code coverage"
fi