# Task 7: Containerization and Deployment Setup - Acceptance Criteria

## Overview
This document defines the acceptance criteria for the containerization and deployment setup implementation. All criteria must be met for the task to be considered complete.

## Functional Requirements

### 1. Dockerfile
- [ ] **File Created**: `Dockerfile` exists in project root
- [ ] **Multi-Stage Build**: Uses separate build and runtime stages
- [ ] **Build Stage**: Uses Rust 1.70+ base image
- [ ] **Dependency Caching**: Implements layer caching for dependencies
- [ ] **Runtime Stage**: Uses debian:bullseye-slim or equivalent
- [ ] **Binary Copy**: Correctly copies compiled binary
- [ ] **Migrations Copy**: Includes database migration files
- [ ] **Port Exposed**: Exposes port 3000
- [ ] **Entrypoint Set**: Defines correct CMD or ENTRYPOINT

### 2. Docker Compose Configuration
- [ ] **File Created**: `docker-compose.yml` exists in project root
- [ ] **Version Specified**: Uses version 3.8 or compatible
- [ ] **PostgreSQL Service**: Database service configured
- [ ] **API Service**: Application service configured
- [ ] **Environment Variables**: DATABASE_URL and RUST_LOG set
- [ ] **Port Mapping**: Maps 3000:3000 for API
- [ ] **Dependencies**: API depends on PostgreSQL
- [ ] **Volumes**: Persistent data volume for PostgreSQL
- [ ] **Development Volumes**: Source code mounted for hot-reload

### 3. Kubernetes Deployment
- [ ] **File Created**: `k8s/deployment.yaml` exists
- [ ] **API Version**: Uses apps/v1
- [ ] **Replicas**: Configured for 3 replicas
- [ ] **Selector Labels**: Proper label selectors defined
- [ ] **Container Image**: References rust-basic-api:latest
- [ ] **Port Configuration**: Container port 3000 exposed
- [ ] **Environment Variables**: DATABASE_URL from secret
- [ ] **Resource Requests**: Memory and CPU requests set
- [ ] **Resource Limits**: Memory and CPU limits defined
- [ ] **Readiness Probe**: HTTP GET to /health configured
- [ ] **Liveness Probe**: HTTP GET to /health configured

### 4. Kubernetes Service
- [ ] **File Created**: `k8s/service.yaml` exists
- [ ] **Service Type**: ClusterIP configured
- [ ] **Selector**: Matches deployment labels
- [ ] **Port Mapping**: Maps port 80 to targetPort 3000
- [ ] **Service Name**: Named rust-basic-api

### 5. Build Script
- [ ] **File Created**: `scripts/build_image.sh` exists
- [ ] **Executable Permission**: Script has execute permissions
- [ ] **Git Integration**: Uses git commit SHA for tagging
- [ ] **Image Building**: Successfully builds Docker image
- [ ] **Tagging Strategy**: Tags with both SHA and latest
- [ ] **Error Handling**: Set -e for error exit
- [ ] **Output Messages**: Provides clear build status

### 6. Deploy Script
- [ ] **File Created**: `scripts/deploy_k8s.sh` exists
- [ ] **Executable Permission**: Script has execute permissions
- [ ] **Namespace Support**: Configurable namespace
- [ ] **Manifest Application**: Applies all k8s manifests
- [ ] **Error Handling**: Set -e for error exit
- [ ] **Status Output**: Reports deployment status

## Technical Requirements

### Docker Image Quality
- [ ] **Build Success**: Image builds without errors
- [ ] **Size Optimization**: Final image under 200MB
- [ ] **No Build Artifacts**: Build stage artifacts not in final image
- [ ] **Security Updates**: Base image packages updated
- [ ] **Non-Root User**: (Recommended) Runs as non-root

### Container Functionality
- [ ] **Container Starts**: Runs without crashes
- [ ] **Port Accessible**: Responds on port 3000
- [ ] **Database Connection**: Connects to PostgreSQL
- [ ] **Environment Config**: Reads environment variables
- [ ] **Graceful Shutdown**: Handles SIGTERM properly

### Kubernetes Compatibility
- [ ] **Valid YAML**: All manifests parse correctly
- [ ] **Resource Validation**: kubectl dry-run passes
- [ ] **Label Consistency**: Labels match across resources
- [ ] **Probe Endpoints**: Health endpoints exist
- [ ] **Secret References**: Database secret referenced correctly

## Test Scenarios

### Scenario 1: Docker Image Build
**Given**: Dockerfile in project root
**When**: Running `docker build -t test-image .`
**Then**:
- Build completes successfully
- Image is created
- Image size is reasonable
- No build warnings

### Scenario 2: Docker Compose Environment
**Given**: docker-compose.yml configured
**When**: Running `docker-compose up -d`
**Then**:
- All services start
- PostgreSQL is accessible
- API responds to requests
- Logs show successful startup

### Scenario 3: Container Health Check
**Given**: Container is running
**When**: Executing `curl http://localhost:3000/health`
**Then**:
- Returns HTTP 200
- Response indicates healthy status
- Database connection verified
- No errors in logs

### Scenario 4: Kubernetes Deployment
**Given**: Kubernetes cluster available
**When**: Running `kubectl apply -f k8s/`
**Then**:
- Deployment created
- 3 pods running
- Service created
- No error events

### Scenario 5: Pod Scaling
**Given**: Deployment is running
**When**: Executing `kubectl scale deployment/rust-basic-api --replicas=5`
**Then**:
- Scales to 5 pods
- All pods become ready
- Service distributes traffic
- No resource issues

### Scenario 6: Build Script Execution
**Given**: Build script exists
**When**: Running `./scripts/build_image.sh`
**Then**:
- Script executes without errors
- Image tagged with git SHA
- Latest tag updated
- Success message displayed

## Edge Cases

### Container Issues
- [ ] **Missing Dependencies**: Clear error for missing packages
- [ ] **Port Conflicts**: Handles port already in use
- [ ] **Database Unavailable**: Retries connection appropriately
- [ ] **Memory Limits**: Respects resource constraints

### Kubernetes Issues
- [ ] **Image Pull Errors**: Clear error messages
- [ ] **Pod Crashes**: Proper restart behavior
- [ ] **Probe Failures**: Pods marked not ready
- [ ] **Resource Exhaustion**: Pods evicted gracefully

### Build Issues
- [ ] **Compilation Failures**: Build fails fast with clear errors
- [ ] **Network Issues**: Handles package download failures
- [ ] **Cache Corruption**: Can rebuild from clean state
- [ ] **Git Not Available**: Handles missing git gracefully

## Validation Checklist

### Manual Testing
1. [ ] Build Docker image - completes successfully
2. [ ] Run container standalone - starts and responds
3. [ ] Run docker-compose up - all services work
4. [ ] Test API endpoints - return expected responses
5. [ ] Apply k8s manifests - deployment successful
6. [ ] Check pod logs - no errors present
7. [ ] Test scaling - pods scale up/down correctly
8. [ ] Run build script - image created and tagged
9. [ ] Run deploy script - manifests applied

### Automated Validation
1. [ ] Docker build in CI pipeline
2. [ ] Container security scanning
3. [ ] Kubernetes manifest validation
4. [ ] Deployment smoke tests
5. [ ] Health probe verification

## Performance Metrics
- **Build Time**: Under 5 minutes for full build
- **Image Size**: Under 200MB for final image
- **Startup Time**: Container ready in < 30 seconds
- **Memory Usage**: Under 256MB per pod
- **CPU Usage**: Under 200m per pod idle

## Security Checklist
- [ ] No secrets in Dockerfile
- [ ] No hardcoded passwords
- [ ] Base image from trusted source
- [ ] Minimal attack surface
- [ ] No unnecessary packages
- [ ] Security scanning passed

## Documentation Requirements
- [ ] Dockerfile comments explain stages
- [ ] docker-compose.yml documented
- [ ] Kubernetes manifests have descriptions
- [ ] Scripts include usage instructions
- [ ] README updated with deployment steps

## Definition of Done
- [ ] All functional requirements met
- [ ] All technical requirements satisfied
- [ ] All test scenarios pass
- [ ] Edge cases handled appropriately
- [ ] Manual testing completed
- [ ] Performance metrics achieved
- [ ] Security checklist passed
- [ ] Documentation complete
- [ ] Scripts are executable
- [ ] Images build and run successfully

## Notes
- Consider adding Helm charts for production deployments
- Implement image scanning in CI/CD pipeline
- Plan for secret rotation strategy
- Consider using distroless images for smaller size
- Add monitoring and logging sidecars in future iterations