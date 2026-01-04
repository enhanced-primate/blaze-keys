# Stage 1: Build the Rust program
FROM docker.io/library/rust:alpine AS builder

RUN apk add musl-dev gcc

# Set the working directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin && echo "fn main() { println!(\"PLACEHOLDER\"); }" >> src/main.rs && cp src/main.rs src/bin/tutorial.rs

# Install dependencies
RUN cargo build --release

# Copy the Rust project files
COPY src/* ./src/
COPY src/bin/* ./src/bin/
COPY example-configs ./example-configs

# Build the Rust program
RUN cargo build --release

# Stage 2: Create the final image with Zsh and Oh My Zsh
FROM docker.io/library/alpine:latest

# Set the working directory
WORKDIR /app

# Copy the pre-built Rust program
COPY --from=builder /app/target/release/blz /app/target/release/tut /app/

# Make the program executable
RUN chmod +x blz

# Create a non-root user
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Install Zsh and Oh My Zsh
RUN apk add zsh zsh-vcs git curl make gcc musl-dev
RUN chown -R appuser:appgroup /app

# Switch to the non-root user
USER appuser

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/robbyrussell/oh-my-zsh/master/tools/install.sh)"

RUN curl -LsSf https://astral.sh/uv/install.sh | sh
RUN source $HOME/.local/bin/env && uv init python_sample

# Configure Oh My Zsh
RUN echo "export PATH=\$PATH:/app" >> ~/.zshrc
RUN echo 'source <(blz --zsh-hook)' >> ~/.zshrc
RUN sed -i 's/robbyrussell/agnoster/g' ~/.zshrc
RUN sed -i 's/plugins.*git.*/plugins=()/g' ~/.zshrc
RUN echo "tut next" >> ~/.zshrc

RUN mkdir -p ~/.config/blaze-keys/
COPY example-configs/templates/global.all.yml /home/appuser/.config/blaze-keys/.blz.yml
COPY res/tutorial/* .

RUN git config --global user.email "test" && git config --global user.name "test" && git init && git commit --allow-empty -m "Initial commit"

# Set the default command to run Zsh
CMD ["zsh"]

