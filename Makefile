.PHONY: run dev build clean test install watch setup

# Run the application in release mode
run:
	cargo run --release

# Development mode with hot reload
dev:
	cargo watch -c -w src -x run

# Build the application
build:
	cargo build

# Build for release
release:
	cargo build --release

# Clean build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# Install dependencies and setup
install:
	cargo build

# Setup development environment
setup:
	@echo "Installing cargo-watch for hot reload..."
	cargo install cargo-watch
	@echo "Setup complete!"

# Watch for changes and run
watch:
	cargo watch -c -w src -x run

# Check code without building
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Lint code
lint:
	cargo clippy

# Start Redis (if using Docker)
redis:
	docker run -d --name redis -p 6379:6379 redis:alpine

# Stop Redis container
redis-stop:
	docker stop redis && docker rm redis

# Start MongoDB (if using Docker)
mongo:
	docker run -d --name mongodb -p 27017:27017 mongo:latest

# Stop MongoDB container
mongo-stop:
	docker stop mongodb && docker rm mongodb

# Start all services (Redis + MongoDB)
services:
	@make mongo
	@make redis
	@echo "MongoDB and Redis are running"

# Stop all services
services-stop:
	@make mongo-stop || true
	@make redis-stop || true
	@echo "Services stopped"

# Help
help:
	@echo "Available commands:"
	@echo "  make run          - Run in release mode"
	@echo "  make dev          - Run with hot reload"
	@echo "  make build        - Build debug version"
	@echo "  make release      - Build release version"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make test         - Run tests"
	@echo "  make setup        - Install cargo-watch"
	@echo "  make check        - Check code"
	@echo "  make fmt          - Format code"
	@echo "  make lint         - Lint code"
	@echo "  make redis        - Start Redis (Docker)"
	@echo "  make mongo        - Start MongoDB (Docker)"
	@echo "  make services     - Start all services"
	@echo "  make services-stop - Stop all services"
