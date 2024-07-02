FROM trustworthysystems/sel4-riscv as build

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static \
    RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --no-modify-path --profile minimal --default-toolchain nightly

# install rust deps for trusted core
COPY rust-toolchain.toml rust-toolchain.toml
RUN rustup target add riscv64gc-unknown-none-elf && \
    cargo install toml-cli cargo-binutils && \
    RUST_VERSION=$(toml get -r rust-toolchain.toml toolchain.channel) && \
    Components=$(toml get -r rust-toolchain.toml toolchain.components | jq -r 'join(" ")') && \
    rustup install $RUST_VERSION && \
    rustup component add --toolchain $RUST_VERSION $Components