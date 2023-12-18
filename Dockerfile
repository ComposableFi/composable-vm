FROM arm64v8/ubuntu:latest

# Update default packages
RUN apt-get update

# Get Ubuntu packages
RUN apt-get install -y \
    build-essential \
    curl \
    bash \
    zip \
    git \
    libfl-dev \
    clang \
    cmake \
    coinor-cbc \
    coinor-libcbc-dev


# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

# Update new packages
RUN apt-get update

ENV PATH="/root/.cargo/bin:${PATH}"

#Get Protobuf
RUN curl -LO https://github.com/protocolbuffers/protobuf/releases/download/v24.2/protoc-24.2-linux-x86_64.zip
RUN unzip protoc-24.2-linux-x86_64.zip -d /usr/local/protoc 
ENV PATH=$PATH:/usr/local/protoc/bin
RUN protoc --version


# Copy the remaining files
COPY . .

# Build mantis
RUN cargo build --bin mantis

ENTRYPOINT ["/bin/bash"]

