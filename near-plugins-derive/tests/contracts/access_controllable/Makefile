universal dev stack
FROM mcr.microsoft.com/devcontainers/universal:2

# Встановлюємо NEAR CLI та NEARAI
RUN npm install -g near-cli \
 && npm install -g nearai \
  && cargo install --locked cargo-watch

  # Встановлюємо залежності для Rust
  RUN rustup update \
   && rustup default stable

   # Перевірка версій
   RUN near --version && nearai --version && node -v && rustc -V && cargo -version