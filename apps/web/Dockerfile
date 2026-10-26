FROM oven/bun:1.4.4-slim AS build
WORKDIR /app
COPY package.json bun.lock bunfig.toml ./
RUN bun install --frozen-lockfile
COPY . .
ARG VITE_API_URL
RUN test -n "$VITE_API_URL" && bun --bun vite build

FROM nginx:1.30.6-alpine-slim
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 80
