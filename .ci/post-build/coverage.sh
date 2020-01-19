# run on osx
docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin
