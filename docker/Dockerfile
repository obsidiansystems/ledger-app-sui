FROM ubuntu:22.04
ENV LANG C.UTF-8

ARG DEBIAN_FRONTEND=noninteractive

ARG LLVM_VERSION=14

RUN apt-get update && apt-get upgrade -qy && \
    apt-get install -qy \
        clang-$LLVM_VERSION \
        clang-tools-$LLVM_VERSION \
        clang-format-$LLVM_VERSION \
        cmake \
        curl \
        doxygen \
        git \
        lcov \
        libbsd-dev \
        libc6-dev-i386 \
        libcmocka0 \
        libcmocka-dev \
        lld-$LLVM_VERSION \
        make \
        protobuf-compiler \
        python-is-python3 \
        python3 \
        python3-pip && \
    apt-get autoclean -y && \
    apt-get autoremove -y && \
    apt-get clean

# Create generic clang & lld symbolic links to their installed version
RUN cd /usr/bin && \
    find . -name "*-"$LLVM_VERSION | sed "s/^\(.*\)\(-"$LLVM_VERSION"\)$/ln -s \1\2 \1/" | sh

# Adding GCC to PATH and defining rustup/cargo home directories
ENV PATH=/opt/gcc-arm-none-eabi-10.3-2021.10/bin:$PATH \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/.cargo

# Install rustup to manage rust toolchains
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y

# Adding cargo binaries to PATH
ENV PATH=${CARGO_HOME}/bin:${PATH}

# Adding ARMV6M target to the default toolchain
RUN rustup target add thumbv6m-none-eabi

RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /app

CMD ["/usr/bin/env", "bash"]
