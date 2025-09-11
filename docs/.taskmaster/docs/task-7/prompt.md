# Task 7: Containerization and Deployment Setup - Autonomous Implementation Prompt

You are a senior DevOps engineer tasked with implementing Docker containerization and Kubernetes deployment configuration for a Rust API project. Your goal is to create a complete container-based deployment pipeline from local development to production-ready Kubernetes manifests.

## Your Mission
Implement comprehensive containerization and deployment infrastructure that enables scalable, portable, and reliable deployments of the Rust API application. This includes creating efficient Docker images, development environments, and production Kubernetes configurations.

## Context
- Project: Rust-based REST API with PostgreSQL database
- Current State: Application development complete (Tasks 1-5)
- Target: Container-based deployment with Kubernetes
- Goal: Enable seamless deployment from development to production

## Implementation Requirements

### 1. Multi-Stage Dockerfile (Project Root)
Create an optimized Dockerfile with build and runtime stages:
```dockerfile
# Build stage
FROM rust:1.70 as builder
WORKDIR /app

# Copy manifests and build dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs for dependency caching
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Remove dummy files
RUN rm -rf src

# Copy actual source code
COPY . .

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy binary and migrations
COPY --from=builder /app/target/release/rust-basic-api /app/rust-basic-api
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3000
CMD ["/app/rust-basic-api"]
```

### 2. Docker Compose Configuration (`docker-compose.yml`)
Set up local development environment with PostgreSQL and API services. Include volume mounting for hot-reload development and proper networking.

### 3. Kubernetes Deployment (`k8s/deployment.yaml`)
Create production deployment manifest with:
- 3 replicas for high availability
- Resource requests and limits
- Health check probes
- Environment configuration via secrets
- Rolling update strategy

### 4. Kubernetes Service (`k8s/service.yaml`)
Define service for internal cluster routing and load balancing across pods.

### 5. Build Script (`scripts/build_image.sh`)
Automate Docker image building with git-based versioning and proper tagging.

### 6. Deploy Script (`scripts/deploy_k8s.sh`)
Automate Kubernetes deployment with namespace management and verification.

## Step-by-Step Implementation

1. **Create Dockerfile**
   - Navigate to project root
   - Create multi-stage Dockerfile
   - Optimize for layer caching
   - Include all necessary runtime dependencies

2. **Set Up Docker Compose**
   - Create docker-compose.yml
   - Configure PostgreSQL service
   - Set up API service with environment
   - Define volumes and networks

3. **Create Kubernetes Manifests**
   - Create k8s directory
   - Write deployment.yaml with full configuration
   - Create service.yaml for networking
   - Consider adding ConfigMap/Secret templates

4. **Implement Build Script**
   - Create scripts/build_image.sh
   - Add git-based versioning
   - Include tagging strategy
   - Make script executable

5. **Create Deploy Script**
   - Write scripts/deploy_k8s.sh
   - Add namespace configuration
   - Include rollout verification
   - Make script executable

## Testing Checklist
- [ ] Dockerfile builds successfully
- [ ] Docker image runs without errors
- [ ] Docker Compose environment starts
- [ ] API accessible via Docker Compose
- [ ] Database connection works in container
- [ ] Kubernetes manifests are valid
- [ ] Deployment creates pods successfully
- [ ] Service routes traffic correctly
- [ ] Health probes pass
- [ ] Scaling works as expected

## Validation Steps
1. Build Docker image: `docker build -t rust-basic-api:test .`
2. Run container: `docker run -p 3000:3000 rust-basic-api:test`
3. Start Docker Compose: `docker-compose up -d`
4. Test health endpoint: `curl http://localhost:3000/health`
5. Validate Kubernetes manifests: `kubectl apply --dry-run=client -f k8s/`
6. Deploy to local cluster: `./scripts/deploy_k8s.sh`
7. Check pod status: `kubectl get pods`
8. Test scaling: `kubectl scale deployment/rust-basic-api --replicas=5`

## Expected Outcomes
- Optimized Docker image under 100MB
- Fast build times with proper caching
- Working local development environment
- Production-ready Kubernetes configuration
- Automated deployment scripts
- Health monitoring implemented
- Scalable architecture

## Configuration Management
- Use environment variables for configuration
- Implement secret management for sensitive data
- Support multiple environments (dev/staging/prod)
- Document all configuration options

## Performance Considerations
- Minimize image size with multi-stage builds
- Use specific base image versions
- Implement proper resource limits
- Configure horizontal pod autoscaling
- Optimize build cache usage

## Security Requirements
- Run containers as non-root user (consider adding)
- Use secrets for database credentials
- Implement network policies (future enhancement)
- Scan images for vulnerabilities
- Use read-only root filesystem where possible

## Error Handling
- Graceful shutdown on SIGTERM
- Proper error logging
- Health check failure handling
- Database connection retry logic
- Container restart policies

## Notes
- Test with local Kubernetes (minikube/kind)
- Consider adding Helm charts for complex deployments
- Document deployment procedures thoroughly
- Plan for zero-downtime deployments
- Consider adding init containers for migrations

Remember to test each component thoroughly in isolation before integrating. The containerization should support both local development and production deployment scenarios efficiently.