# Build frontend
FROM node:20 AS frontend-builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# Build backend
FROM rust:1.83-bullseye AS backend-builder
WORKDIR /app
COPY backend/ ./
COPY --from=frontend-builder /app/dist ./www
RUN ls -l
RUN cargo build --release

# Merge frontend and backend
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3
WORKDIR /root/
COPY --from=backend-builder /app/target/release/secret-santa ./
COPY --from=frontend-builder /app/dist ./www
RUN sqlite3 /root/santa.db "VACUUM;"
ENV DATABASE_URL="sqlite:/root/santa.db"
RUN ls -l
EXPOSE 8080
CMD ["./secret-santa"]