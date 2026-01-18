# Build frontend
FROM node:22-alpine AS frontend-builder
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY . .
ARG API_URL=/api
ENV VITE_API_URL=$API_URL
RUN pnpm build

# Build server
FROM node:22-alpine AS server-builder
WORKDIR /app/server
COPY server/package.json server/pnpm-lock.yaml* ./
RUN corepack enable && pnpm install --frozen-lockfile
COPY server/ .
RUN pnpm build

# Production
FROM node:22-alpine AS production
WORKDIR /app

COPY --from=server-builder /app/server/dist ./dist
COPY --from=server-builder /app/server/node_modules ./node_modules
COPY --from=server-builder /app/server/package.json ./
COPY --from=frontend-builder /app/dist ./public

ENV NODE_ENV=production
ENV PORT=3000
ENV ROMS_DIR=/roms

EXPOSE 3000

CMD ["node", "dist/index.js"]
