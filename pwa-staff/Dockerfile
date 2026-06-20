FROM node:20-alpine

WORKDIR /app

# Copy manifests first for layer caching
COPY package.json package-lock.json ./

RUN npm ci

# Copy source
COPY . .

# Build: esbuild TS + postcss CSS → dist/
RUN npm run build

EXPOSE 3000

# Serve the compiled dist/ directory.
# `npx serve` is a zero-dep static server; -s enables SPA fallback, -l sets port.
CMD ["npx", "serve", "-s", "dist", "-l", "3000"]
