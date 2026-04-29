# Phase 7: DevOps, Deployment & Security Engineer

## Overview
This phase establishes the production infrastructure, CI/CD pipelines, security protocols, and deployment strategies for the ride-hailing platform. It ensures scalable, secure, and reliable operations.

---

## 1. Infrastructure Architecture

### 1.1 Cloud Infrastructure (AWS)

#### Core Services
```yaml
Infrastructure:
  Compute:
    - ECS/Fargate: Container orchestration for microservices
    - Lambda: Serverless functions for event-driven tasks
    - EC2: Bastion hosts and specialized workloads
  
  Storage:
    - RDS PostgreSQL: Primary database with read replicas
    - ElastiCache Redis: Session management and caching
    - S3: Media storage, backups, logs
    - DynamoDB: Real-time location data, trip metadata
  
  Networking:
    - VPC: Isolated network with public/private subnets
    - Application Load Balancer: Traffic distribution
    - CloudFront: CDN for static assets
    - Route53: DNS management
    - API Gateway: RESTful API endpoint management
  
  Monitoring:
    - CloudWatch: Logs, metrics, alarms
    - X-Ray: Distributed tracing
    - Datadog/New Relic: APM integration
```

### 1.2 Infrastructure as Code (Terraform)

#### Directory Structure
```
infrastructure/
├── modules/
│   ├── vpc/
│   ├── ecs/
│   ├── rds/
│   ├── redis/
│   ├── s3/
│   └── monitoring/
├── environments/
│   ├── development/
│   ├── staging/
│   └── production/
├── scripts/
│   ├── bootstrap.sh
│   └── migrate.sh
└── terraform.tfvars
```

#### Main Terraform Configuration
```hcl
# infrastructure/environments/production/main.tf

terraform {
  required_version = ">= 1.5.0"
  
  backend "s3" {
    bucket         = "ride-hailing-terraform-state"
    key            = "production/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-locks"
  }
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "ride-hailing"
      Environment = "production"
      ManagedBy   = "terraform"
    }
  }
}

# VPC Module
module "vpc" {
  source = "../../modules/vpc"
  
  vpc_cidr           = var.vpc_cidr
  availability_zones = var.availability_zones
  environment        = var.environment
}

# ECS Cluster
module "ecs" {
  source = "../../modules/ecs"
  
  cluster_name      = "${var.project_name}-${var.environment}"
  vpc_id            = module.vpc.vpc_id
  private_subnets   = module.vpc.private_subnets
  security_group_id = module.vpc.default_sg_id
  
  services = [
    {
      name          = "api-gateway"
      container_image = "aws.ecr.repository/api-gateway:latest"
      cpu           = 512
      memory        = 1024
      desired_count = 3
    },
    {
      name          = "user-service"
      container_image = "aws.ecr.repository/user-service:latest"
      cpu           = 256
      memory        = 512
      desired_count = 2
    },
    {
      name          = "trip-service"
      container_image = "aws.ecr.repository/trip-service:latest"
      cpu           = 512
      memory        = 1024
      desired_count = 3
    },
    {
      name          = "payment-service"
      container_image = "aws.ecr.repository/payment-service:latest"
      cpu           = 512
      memory        = 1024
      desired_count = 2
    },
    {
      name          = "notification-service"
      container_image = "aws.ecr.repository/notification-service:latest"
      cpu           = 256
      memory        = 512
      desired_count = 2
    },
    {
      name          = "location-service"
      container_image = "aws.ecr.repository/location-service:latest"
      cpu           = 1024
      memory        = 2048
      desired_count = 3
    }
  ]
}

# RDS PostgreSQL
module "database" {
  source = "../../modules/rds"
  
  identifier           = "${var.project_name}-db"
  engine              = "postgres"
  engine_version      = "15.4"
  instance_class      = "db.r6g.large"
  allocated_storage   = 100
  multi_az            = true
  vpc_id              = module.vpc.vpc_id
  private_subnets     = module.vpc.private_subnets
  backup_retention    = 30
  environment         = var.environment
}

# ElastiCache Redis
module "redis" {
  source = "../../modules/redis"
  
  cluster_id          = "${var.project_name}-redis"
  node_type           = "cache.r6g.large"
  num_cache_nodes     = 2
  vpc_id              = module.vpc.vpc_id
  private_subnets     = module.vpc.private_subnets
  environment         = var.environment
}
```

---

## 2. CI/CD Pipeline

### 2.1 GitHub Actions Workflow

#### Main CI/CD Pipeline
```yaml
# .github/workflows/deploy.yml

name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  AWS_REGION: us-east-1
  ECR_REPOSITORY: ride-hailing
  ECS_CLUSTER: ride-hailing-production

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run linting
        run: npm run lint
      
      - name: Run unit tests
        run: npm run test:unit
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/postgres
          REDIS_URL: redis://localhost:6379
      
      - name: Run integration tests
        run: npm run test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/postgres
          REDIS_URL: redis://localhost:6379
          JWT_SECRET: test-secret
          STRIPE_SECRET_KEY: sk_test_xxx
      
      - name: Upload coverage reports
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
          flags: backend

  build:
    needs: test
    runs-on: ubuntu-latest
    
    outputs:
      image_tag: ${{ steps.build.outputs.image_tag }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}
      
      - name: Login to Amazon ECR
        id: login-ecr
        uses: aws-actions/amazon-ecr-login@v2
      
      - name: Build, tag, and push image to Amazon ECR
        id: build
        env:
          ECR_REGISTRY: ${{ steps.login-ecr.outputs.registry }}
          IMAGE_TAG: ${{ github.sha }}
        run: |
          docker build -t $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG .
          docker push $ECR_REGISTRY/$ECR_REPOSITORY:$IMAGE_TAG
          echo "image_tag=$IMAGE_TAG" >> $GITHUB_OUTPUT

  deploy-staging:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/develop'
    environment: staging
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}
      
      - name: Download task definition
        run: |
          aws ecs describe-task-definition \
            --task-definition ride-hailing-staging \
            --query taskDefinition > task-definition.json
      
      - name: Render new task definition
        id: render
        uses: aws-actions/amazon-ecs-render-task-definition@v1
        with:
          task-definition: task-definition.json
          container-name: api-service
          image: ${{ secrets.ECR_REGISTRY }}/${{ env.ECR_REPOSITORY }}:${{ needs.build.outputs.image_tag }}
      
      - name: Deploy to Amazon ECS
        uses: aws-actions/amazon-ecs-deploy-task-definition@v1
        with:
          task-definition: ${{ steps.render.outputs.task-definition }}
          service: ride-hailing-api-staging
          cluster: ride-hailing-staging
          wait-for-service-stability: true

  deploy-production:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: production
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Configure AWS credentials
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: ${{ env.AWS_REGION }}
      
      - name: Download task definition
        run: |
          aws ecs describe-task-definition \
            --task-definition ride-hailing-production \
            --query taskDefinition > task-definition.json
      
      - name: Render new task definition
        id: render
        uses: aws-actions/amazon-ecs-render-task-definition@v1
        with:
          task-definition: task-definition.json
          container-name: api-service
          image: ${{ secrets.ECR_REGISTRY }}/${{ env.ECR_REPOSITORY }}:${{ needs.build.outputs.image_tag }}
      
      - name: Deploy to Amazon ECS (Blue-Green)
        uses: aws-actions/amazon-ecs-deploy-task-definition@v1
        with:
          task-definition: ${{ steps.render.outputs.task-definition }}
          service: ride-hailing-api-production
          cluster: ride-hailing-production
          codedeploy-appspec: appspec.yaml
          codedeploy-application: ride-hailing-production
          codedeploy-deployment-group: production-deployment
          wait-for-service-stability: true
          wait-for-minutes: 30

  security-scan:
    needs: build
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ secrets.ECR_REGISTRY }}/${{ env.ECR_REPOSITORY }}:${{ needs.build.outputs.image_tag }}
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'
      
      - name: Upload Trivy scan results to GitHub Security tab
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
```

### 2.2 Deployment Strategy

#### Blue-Green Deployment Configuration
```yaml
# appspec.yaml

version: 0.0
Resources:
  - TargetService:
      Type: AWS::ECS::Service
      Properties:
        TaskDefinition: "arn:aws:ecs:us-east-1:123456789:task-definition/ride-hailing-production:latest"
        LoadBalancerInfo:
          ContainerName: "api-service"
          ContainerPort: 3000
        
DeploymentConfiguration:
  MinimumHealthyPercent: 100
  MaximumPercent: 200
  Alarms:
    - Name: ProductionErrorRate
    - Name: ProductionLatency
    - Name: ProductionHealthChecks
```

---

## 3. Security Implementation

### 3.1 Authentication & Authorization

#### JWT Token Management
```typescript
// src/security/jwt.service.ts

import { sign, verify, decode } from 'jsonwebtoken';
import { redisClient } from '../config/redis';
import { createHash } from 'crypto';

interface JWTPayload {
  userId: string;
  userType: 'rider' | 'driver';
  email: string;
  iat?: number;
  exp?: number;
}

interface TokenPair {
  accessToken: string;
  refreshToken: string;
}

export class JWTService {
  private readonly accessSecret: string;
  private readonly refreshSecret: string;
  private readonly accessExpiry: string;
  private readonly refreshExpiry: string;
  
  constructor() {
    this.accessSecret = process.env.JWT_ACCESS_SECRET!;
    this.refreshSecret = process.env.JWT_REFRESH_SECRET!;
    this.accessExpiry = process.env.JWT_ACCESS_EXPIRY || '15m';
    this.refreshExpiry = process.env.JWT_REFRESH_EXPIRY || '7d';
  }
  
  generateTokenPair(payload: Omit<JWTPayload, 'iat' | 'exp'>): TokenPair {
    const accessToken = sign(payload, this.accessSecret, {
      expiresIn: this.accessExpiry,
    });
    
    const refreshToken = sign(payload, this.refreshSecret, {
      expiresIn: this.refreshExpiry,
    });
    
    // Store refresh token in Redis
    this.storeRefreshToken(payload.userId, refreshToken);
    
    return { accessToken, refreshToken };
  }
  
  verifyAccessToken(token: string): JWTPayload {
    try {
      return verify(token, this.accessSecret) as JWTPayload;
    } catch (error) {
      throw new Error('Invalid or expired access token');
    }
  }
  
  async verifyRefreshToken(userId: string, token: string): Promise<JWTPayload> {
    try {
      // Verify signature
      const payload = verify(token, this.refreshSecret) as JWTPayload;
      
      // Check if token is blacklisted
      const isBlacklisted = await this.isTokenBlacklisted(token);
      if (isBlacklisted) {
        throw new Error('Token has been revoked');
      }
      
      // Verify token exists in Redis
      const storedToken = await redisClient.get(`refresh_token:${userId}`);
      if (!storedToken || storedToken !== token) {
        throw new Error('Invalid refresh token');
      }
      
      return payload;
    } catch (error) {
      throw new Error('Invalid or expired refresh token');
    }
  }
  
  async revokeTokens(userId: string): Promise<void> {
    const currentToken = await redisClient.get(`refresh_token:${userId}`);
    if (currentToken) {
      // Add to blacklist with TTL matching remaining token life
      const decoded = decode(currentToken) as JWTPayload;
      const ttl = decoded.exp! * 1000 - Date.now();
      
      if (ttl > 0) {
        await redisClient.setEx(
          `blacklist:${currentToken}`,
          Math.ceil(ttl / 1000),
          'revoked'
        );
      }
      
      await redisClient.del(`refresh_token:${userId}`);
    }
  }
  
  private async storeRefreshToken(userId: string, token: string): Promise<void> {
    const decoded = decode(token) as JWTPayload;
    const ttl = decoded.exp! * 1000 - Date.now();
    
    await redisClient.setEx(
      `refresh_token:${userId}`,
      Math.ceil(ttl / 1000),
      token
    );
  }
  
  private async isTokenBlacklisted(token: string): Promise<boolean> {
    const result = await redisClient.get(`blacklist:${token}`);
    return result === 'revoked';
  }
  
  async rotateTokens(refreshToken: string): Promise<TokenPair> {
    const payload = await this.verifyRefreshToken(
      (decode(refreshToken) as JWTPayload).userId,
      refreshToken
    );
    
    // Revoke old tokens
    await this.revokeTokens(payload.userId);
    
    // Generate new token pair
    return this.generateTokenPair({
      userId: payload.userId,
      userType: payload.userType,
      email: payload.email,
    });
  }
}
```

#### Role-Based Access Control (RBAC)
```typescript
// src/middleware/auth.middleware.ts

import { Request, Response, NextFunction } from 'express';
import { JWTService } from '../security/jwt.service';
import { AppError } from '../utils/errors';

export enum UserRole {
  RIDER = 'rider',
  DRIVER = 'driver',
  ADMIN = 'admin',
}

export interface AuthRequest extends Request {
  user?: {
    userId: string;
    userType: UserRole;
    email: string;
  };
}

export class AuthMiddleware {
  private jwtService: JWTService;
  
  constructor() {
    this.jwtService = new JWTService();
  }
  
  authenticate = async (
    req: AuthRequest,
    res: Response,
    next: NextFunction
  ): Promise<void> => {
    try {
      const authHeader = req.headers.authorization;
      
      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        throw new AppError('Authorization header missing or invalid', 401);
      }
      
      const token = authHeader.split(' ')[1];
      const payload = this.jwtService.verifyAccessToken(token);
      
      req.user = {
        userId: payload.userId,
        userType: payload.userType as UserRole,
        email: payload.email,
      };
      
      next();
    } catch (error) {
      if (error instanceof AppError) {
        next(error);
      } else {
        next(new AppError('Authentication failed', 401));
      }
    }
  };
  
  authorize(...roles: UserRole[]) {
    return (req: AuthRequest, res: Response, next: NextFunction): void => {
      if (!req.user) {
        return next(new AppError('User not authenticated', 401));
      }
      
      if (!roles.includes(req.user.userType)) {
        return next(
          new AppError('Insufficient permissions for this action', 403)
        );
      }
      
      next();
    };
  }
  
  optionalAuth = async (
    req: AuthRequest,
    res: Response,
    next: NextFunction
  ): Promise<void> => {
    try {
      const authHeader = req.headers.authorization;
      
      if (authHeader && authHeader.startsWith('Bearer ')) {
        const token = authHeader.split(' ')[1];
        const payload = this.jwtService.verifyAccessToken(token);
        
        req.user = {
          userId: payload.userId,
          userType: payload.userType as UserRole,
          email: payload.email,
        };
      }
      
      next();
    } catch (error) {
      // Continue without authentication
      next();
    }
  };
}
```

### 3.2 Data Encryption

#### Encryption Service
```typescript
// src/security/encryption.service.ts

import { createCipheriv, createDecipheriv, randomBytes, scrypt } from 'crypto';
import { promisify } from 'util';

const scryptAsync = promisify(scrypt);

export class EncryptionService {
  private readonly algorithm: string = 'aes-256-gcm';
  private readonly keyLength: number = 32;
  private readonly ivLength: number = 16;
  private readonly authTagLength: number = 16;
  private readonly saltLength: number = 16;
  
  async encrypt(plainText: string, password: string): Promise<string> {
    const salt = randomBytes(this.saltLength);
    const key = (await scryptAsync(password, salt, this.keyLength)) as Buffer;
    const iv = randomBytes(this.ivLength);
    
    const cipher = createCipheriv(this.algorithm, key, iv);
    
    let encrypted = cipher.update(plainText, 'utf8', 'hex');
    encrypted += cipher.final('hex');
    
    const authTag = cipher.getAuthTag().toString('hex');
    
    // Combine salt, IV, auth tag, and encrypted data
    return `${salt.toString('hex')}:${iv.toString('hex')}:${authTag}:${encrypted}`;
  }
  
  async decrypt(encryptedData: string, password: string): Promise<string> {
    const [saltHex, ivHex, authTagHex, encrypted] = encryptedData.split(':');
    
    const salt = Buffer.from(saltHex, 'hex');
    const iv = Buffer.from(ivHex, 'hex');
    const authTag = Buffer.from(authTagHex, 'hex');
    
    const key = (await scryptAsync(password, salt, this.keyLength)) as Buffer;
    
    const decipher = createDecipheriv(this.algorithm, key, iv);
    decipher.setAuthTag(authTag);
    
    let decrypted = decipher.update(encrypted, 'hex', 'utf8');
    decrypted += decipher.final('utf8');
    
    return decrypted;
  }
  
  hashPassword(password: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const salt = randomBytes(16).toString('hex');
      
      scrypt(password, salt, 64, (err, derivedKey) => {
        if (err) reject(err);
        resolve(`${salt}:${derivedKey.toString('hex')}`);
      });
    });
  }
  
  verifyPassword(password: string, hash: string): Promise<boolean> {
    return new Promise((resolve, reject) => {
      const [salt, keyHex] = hash.split(':');
      
      scrypt(password, salt, 64, (err, derivedKey) => {
        if (err) reject(err);
        resolve(derivedKey.toString('hex') === keyHex);
      });
    });
  }
}
```

#### Field-Level Encryption for Sensitive Data
```typescript
// src/decorators/encrypted-field.decorator.ts

import { EncryptionService } from '../security/encryption.service';

const encryptionService = new EncryptionService();
const ENCRYPTION_KEY = process.env.DATA_ENCRYPTION_KEY!;

export function EncryptedField(): PropertyDecorator {
  return (target: any, propertyKey: string | symbol) => {
    const privateKey = `__${String(propertyKey)}_encrypted`;
    
    Object.defineProperty(target, propertyKey, {
      get() {
        const encryptedValue = this[privateKey];
        if (!encryptedValue) return undefined;
        
        return encryptionService.decrypt(encryptedValue, ENCRYPTION_KEY);
      },
      set(value: any) {
        if (value === undefined || value === null) {
          this[privateKey] = undefined;
          return;
        }
        
        this[privateKey] = encryptionService.encrypt(
          String(value),
          ENCRYPTION_KEY
        );
      },
      enumerable: true,
      configurable: true,
    });
  };
}

// Usage example
export class PaymentMethod {
  @EncryptedField()
  cardNumber!: string;
  
  @EncryptedField()
  cvv!: string;
  
  @EncryptedField()
  billingAddress!: string;
}
```

### 3.3 API Security

#### Rate Limiting
```typescript
// src/middleware/rate-limit.middleware.ts

import { Request, Response, NextFunction } from 'express';
import { redisClient } from '../config/redis';
import { AppError } from '../utils/errors';

interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  message: string;
}

export class RateLimitMiddleware {
  private config: RateLimitConfig;
  
  constructor(config: RateLimitConfig) {
    this.config = config;
  }
  
  limit = async (
    req: Request,
    res: Response,
    next: NextFunction
  ): Promise<void> => {
    try {
      const identifier = this.getIdentifier(req);
      const key = `ratelimit:${identifier}`;
      const windowMs = this.config.windowMs;
      
      const current = await redisClient.incr(key);
      
      if (current === 1) {
        await redisClient.expire(key, Math.ceil(windowMs / 1000));
      }
      
      const remaining = Math.max(0, this.config.maxRequests - current);
      
      res.setHeader('X-RateLimit-Limit', this.config.maxRequests);
      res.setHeader('X-RateLimit-Remaining', remaining);
      res.setHeader('X-RateLimit-Reset', Date.now() + windowMs);
      
      if (current > this.config.maxRequests) {
        throw new AppError(this.config.message, 429);
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
  
  private getIdentifier(req: Request): string {
    // Use API key if present
    const apiKey = req.headers['x-api-key'];
    if (apiKey) {
      return `apikey:${apiKey}`;
    }
    
    // Use user ID if authenticated
    if ((req as any).user?.userId) {
      return `user:${(req as any).user.userId}`;
    }
    
    // Fall back to IP address
    const ip = req.ip || req.socket.remoteAddress || 'unknown';
    return `ip:${ip}`;
  }
}

// Pre-configured rate limiters
export const rateLimiters = {
  general: new RateLimitMiddleware({
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 100,
    message: 'Too many requests, please try again later',
  }),
  
  auth: new RateLimitMiddleware({
    windowMs: 15 * 60 * 1000, // 15 minutes
    maxRequests: 5,
    message: 'Too many authentication attempts, please try again later',
  }),
  
  sms: new RateLimitMiddleware({
    windowMs: 60 * 60 * 1000, // 1 hour
    maxRequests: 10,
    message: 'SMS limit exceeded, please try again later',
  }),
  
  tripCreation: new RateLimitMiddleware({
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 5,
    message: 'Too many trip creation attempts',
  }),
};
```

#### Input Validation & Sanitization
```typescript
// src/middleware/validation.middleware.ts

import { Request, Response, NextFunction } from 'express';
import { z, ZodSchema } from 'zod';
import { AppError } from '../utils/errors';
import sanitizeHtml from 'sanitize-html';

export class ValidationMiddleware {
  static validate(schema: ZodSchema) {
    return (req: Request, res: Response, next: NextFunction): void => {
      try {
        schema.parse(req.body);
        next();
      } catch (error) {
        if (error instanceof z.ZodError) {
          const errors = error.errors.map((e) => ({
            field: e.path.join('.'),
            message: e.message,
          }));
          
          next(new AppError('Validation failed', 400, errors));
        } else {
          next(error);
        }
      }
    };
  }
  
  static sanitizeHTML(fields: string[]) {
    return (req: Request, res: Response, next: NextFunction): void => {
      fields.forEach((field) => {
        if (req.body[field] && typeof req.body[field] === 'string') {
          req.body[field] = sanitizeHtml(req.body[field], {
            allowedTags: [],
            allowedAttributes: {},
          });
        }
      });
      
      next();
    };
  }
  
  static validateQuery(schema: ZodSchema) {
    return (req: Request, res: Response, next: NextFunction): void => {
      try {
        req.query = schema.parse(req.query);
        next();
      } catch (error) {
        if (error instanceof z.ZodError) {
          const errors = error.errors.map((e) => ({
            field: e.path.join('.'),
            message: e.message,
          }));
          
          next(new AppError('Invalid query parameters', 400, errors));
        } else {
          next(error);
        }
      }
    };
  }
}

// Example schemas
export const createTripSchema = z.object({
  pickupLocation: z.object({
    latitude: z.number().min(-90).max(90),
    longitude: z.number().min(-180).max(180),
    address: z.string().min(1).max(500),
  }),
  dropoffLocation: z.object({
    latitude: z.number().min(-90).max(90),
    longitude: z.number().min(-180).max(180),
    address: z.string().min(1).max(500),
  }),
  vehicleType: z.enum(['economy', 'comfort', 'premium', 'xl']),
  paymentMethodId: z.string().uuid(),
});
```

### 3.4 Security Headers & CORS

```typescript
// src/middleware/security.middleware.ts

import { Request, Response, NextFunction } from 'express';
import helmet from 'helmet';
import cors from 'cors';

export class SecurityMiddleware {
  static applySecurityHeaders() {
    return helmet({
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          scriptSrc: ["'self'"],
          styleSrc: ["'self'", "'unsafe-inline'"],
          imgSrc: ["'self'", 'data:', 'https:'],
          connectSrc: ["'self'", 'https://api.stripe.com'],
          fontSrc: ["'self'"],
          objectSrc: ["'none'"],
          mediaSrc: ["'self'"],
          frameSrc: ["'none'"],
        },
      },
      crossOriginEmbedderPolicy: true,
      crossOriginOpenerPolicy: true,
      crossOriginResourcePolicy: { policy: 'same-site' },
      dnsPrefetchControl: { allow: false },
      frameguard: { action: 'deny' },
      hidePoweredBy: true,
      hsts: {
        maxAge: 31536000,
        includeSubDomains: true,
        preload: true,
      },
      ieNoOpen: true,
      noSniff: true,
      originAgentCluster: true,
      permittedCrossDomainPolicies: { permittedPolicies: 'none' },
      referrerPolicy: { policy: 'strict-origin-when-cross-origin' },
      xssFilter: true,
    });
  }
  
  static configureCORS() {
    return cors({
      origin: (origin, callback) => {
        const allowedOrigins = process.env.ALLOWED_ORIGINS?.split(',') || [];
        
        // Allow requests with no origin (mobile apps, curl, etc.)
        if (!origin) {
          return callback(null, true);
        }
        
        if (allowedOrigins.includes(origin)) {
          callback(null, true);
        } else {
          callback(new Error('Not allowed by CORS'));
        }
      },
      credentials: true,
      methods: ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'OPTIONS'],
      allowedHeaders: [
        'Content-Type',
        'Authorization',
        'X-Requested-With',
        'X-API-Key',
      ],
      exposedHeaders: [
        'X-RateLimit-Limit',
        'X-RateLimit-Remaining',
        'X-RateLimit-Reset',
      ],
      maxAge: 86400, // 24 hours
    });
  }
}
```

---

## 4. Monitoring & Observability

### 4.1 Application Monitoring

#### Custom Metrics & Health Checks
```typescript
// src/monitoring/metrics.service.ts

import { Counter, Gauge, Histogram, Summary } from 'prom-client';
import { EventEmitter } from 'events';

export class MetricsService {
  private static instance: MetricsService;
  private eventEmitter: EventEmitter;
  
  // Counters
  private httpRequestTotal: Counter;
  private httpErrorsTotal: Counter;
  private tripCreatedTotal: Counter;
  private tripCompletedTotal: Counter;
  private tripCancelledTotal: Counter;
  private paymentProcessedTotal: Counter;
  private paymentFailedTotal: Counter;
  
  // Gauges
  private activeDriversGauge: Gauge;
  private activeTripsGauge: Gauge;
  private queueSizeGauge: Gauge;
  private databaseConnectionsGauge: Gauge;
  
  // Histograms
  private requestDurationHistogram: Histogram;
  private tripDurationHistogram: Histogram;
  private paymentProcessingTimeHistogram: Histogram;
  
  // Summaries
  private apiLatencySummary: Summary;
  
  private constructor() {
    this.eventEmitter = new EventEmitter();
    this.initializeMetrics();
  }
  
  static getInstance(): MetricsService {
    if (!MetricsService.instance) {
      MetricsService.instance = new MetricsService();
    }
    return MetricsService.instance;
  }
  
  private initializeMetrics(): void {
    // HTTP Request Counter
    this.httpRequestTotal = new Counter({
      name: 'http_requests_total',
      help: 'Total number of HTTP requests',
      labelNames: ['method', 'route', 'status_code'],
    });
    
    // HTTP Errors Counter
    this.httpErrorsTotal = new Counter({
      name: 'http_errors_total',
      help: 'Total number of HTTP errors',
      labelNames: ['method', 'route', 'error_type'],
    });
    
    // Trip Metrics
    this.tripCreatedTotal = new Counter({
      name: 'trips_created_total',
      help: 'Total number of trips created',
      labelNames: ['vehicle_type'],
    });
    
    this.tripCompletedTotal = new Counter({
      name: 'trips_completed_total',
      help: 'Total number of trips completed',
      labelNames: ['vehicle_type'],
    });
    
    this.tripCancelledTotal = new Counter({
      name: 'trips_cancelled_total',
      help: 'Total number of trips cancelled',
      labelNames: ['reason'],
    });
    
    // Payment Metrics
    this.paymentProcessedTotal = new Counter({
      name: 'payments_processed_total',
      help: 'Total number of payments processed',
      labelNames: ['status', 'payment_method'],
    });
    
    this.paymentFailedTotal = new Counter({
      name: 'payments_failed_total',
      help: 'Total number of failed payments',
      labelNames: ['reason'],
    });
    
    // Active Drivers Gauge
    this.activeDriversGauge = new Gauge({
      name: 'active_drivers',
      help: 'Number of currently active drivers',
      labelNames: ['city', 'vehicle_type'],
    });
    
    // Active Trips Gauge
    this.activeTripsGauge = new Gauge({
      name: 'active_trips',
      help: 'Number of currently active trips',
    });
    
    // Queue Size Gauge
    this.queueSizeGauge = new Gauge({
      name: 'queue_size',
      help: 'Current size of various queues',
      labelNames: ['queue_name'],
    });
    
    // Database Connections Gauge
    this.databaseConnectionsGauge = new Gauge({
      name: 'database_connections',
      help: 'Number of active database connections',
      labelNames: ['pool_name', 'type'],
    });
    
    // Request Duration Histogram
    this.requestDurationHistogram = new Histogram({
      name: 'http_request_duration_seconds',
      help: 'HTTP request duration in seconds',
      labelNames: ['method', 'route'],
      buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
    });
    
    // Trip Duration Histogram
    this.tripDurationHistogram = new Histogram({
      name: 'trip_duration_seconds',
      help: 'Trip duration in seconds',
      labelNames: ['vehicle_type'],
      buckets: [60, 300, 600, 1200, 1800, 3600, 7200],
    });
    
    // Payment Processing Time Histogram
    this.paymentProcessingTimeHistogram = new Histogram({
      name: 'payment_processing_time_seconds',
      help: 'Payment processing time in seconds',
      labelNames: ['payment_method'],
      buckets: [0.1, 0.5, 1, 2, 5, 10],
    });
    
    // API Latency Summary
    this.apiLatencySummary = new Summary({
      name: 'api_latency_seconds',
      help: 'API latency in seconds',
      labelNames: ['endpoint'],
      percentiles: [0.5, 0.9, 0.95, 0.99],
    });
  }
  
  // Metric recording methods
  recordHttpRequest(method: string, route: string, statusCode: number): void {
    this.httpRequestTotal.inc({ method, route, status_code: statusCode });
  }
  
  recordHttpError(method: string, route: string, errorType: string): void {
    this.httpErrorsTotal.inc({ method, route, error_type: errorType });
  }
  
  recordTripCreated(vehicleType: string): void {
    this.tripCreatedTotal.inc({ vehicle_type: vehicleType });
  }
  
  recordTripCompleted(vehicleType: string, durationSeconds: number): void {
    this.tripCompletedTotal.inc({ vehicle_type: vehicleType });
    this.tripDurationHistogram.observe(
      { vehicle_type: vehicleType },
      durationSeconds
    );
  }
  
  recordTripCancelled(reason: string): void {
    this.tripCancelledTotal.inc({ reason });
  }
  
  recordPaymentProcessed(
    status: string,
    paymentMethod: string,
    processingTimeSeconds: number
  ): void {
    this.paymentProcessedTotal.inc({ status, payment_method: paymentMethod });
    this.paymentProcessingTimeHistogram.observe(
      { payment_method: paymentMethod },
      processingTimeSeconds
    );
  }
  
  recordPaymentFailed(reason: string): void {
    this.paymentFailedTotal.inc({ reason });
  }
  
  updateActiveDrivers(count: number, city: string, vehicleType: string): void {
    this.activeDriversGauge.set({ city, vehicle_type: vehicleType }, count);
  }
  
  updateActiveTrips(count: number): void {
    this.activeTripsGauge.set(count);
  }
  
  updateQueueSize(queueName: string, size: number): void {
    this.queueSizeGauge.set({ queue_name: queueName }, size);
  }
  
  updateDatabaseConnections(
    poolName: string,
    type: string,
    count: number
  ): void {
    this.databaseConnectionsGauge.set({ pool_name: poolName, type }, count);
  }
  
  recordRequestDuration(
    method: string,
    route: string,
    durationSeconds: number
  ): void {
    this.requestDurationHistogram.observe(
      { method, route },
      durationSeconds
    );
  }
  
  recordApiLatency(endpoint: string, latencySeconds: number): void {
    this.apiLatencySummary.observe({ endpoint }, latencySeconds);
  }
  
  getMetrics(): Promise<string> {
    return Promise.resolve();
  }
}
```

#### Health Check Endpoints
```typescript
// src/routes/health.routes.ts

import { Router, Request, Response } from 'express';
import { sequelize } from '../config/database';
import { redisClient } from '../config/redis';
import { MetricsService } from '../monitoring/metrics.service';

const router = Router();

interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'degraded';
  timestamp: string;
  version: string;
  uptime: number;
  checks: {
    database?: {
      status: 'up' | 'down';
      responseTime?: number;
    };
    redis?: {
      status: 'up' | 'down';
      responseTime?: number;
    };
    memory?: {
      status: 'up' | 'warning' | 'critical';
      usage: number;
      total: number;
      percentage: number;
    };
    disk?: {
      status: 'up' | 'warning' | 'critical';
      usage: number;
      total: number;
      percentage: number;
    };
  };
}

router.get('/live', (req: Request, res: Response) => {
  // Liveness probe - just checking if server is running
  res.json({
    status: 'alive',
    timestamp: new Date().toISOString(),
    version: process.env.APP_VERSION || 'unknown',
  });
});

router.get('/ready', async (req: Request, res: Response) => {
  // Readiness probe - checking if all dependencies are available
  const health: HealthStatus = {
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: process.env.APP_VERSION || 'unknown',
    uptime: process.uptime(),
    checks: {},
  };
  
  // Check database
  try {
    const start = Date.now();
    await sequelize.query('SELECT 1');
    const responseTime = Date.now() - start;
    
    health.checks.database = {
      status: 'up',
      responseTime,
    };
  } catch (error) {
    health.checks.database = { status: 'down' };
    health.status = 'unhealthy';
  }
  
  // Check Redis
  try {
    const start = Date.now();
    await redisClient.ping();
    const responseTime = Date.now() - start;
    
    health.checks.redis = {
      status: 'up',
      responseTime,
    };
  } catch (error) {
    health.checks.redis = { status: 'down' };
    health.status = 'unhealthy';
  }
  
  // Check memory
  const memoryUsage = process.memoryUsage();
  const memoryPercentage = (memoryUsage.heapUsed / memoryUsage.heapTotal) * 100;
  
  health.checks.memory = {
    status: memoryPercentage > 90 ? 'critical' : memoryPercentage > 80 ? 'warning' : 'up',
    usage: memoryUsage.heapUsed,
    total: memoryUsage.heapTotal,
    percentage: memoryPercentage,
  };
  
  if (health.checks.memory.status === 'critical') {
    health.status = 'degraded';
  }
  
  const statusCode = health.status === 'healthy' ? 200 : 
                     health.status === 'degraded' ? 200 : 503;
  
  res.status(statusCode).json(health);
});

router.get('/metrics', async (req: Request, res: Response) => {
  const metricsService = MetricsService.getInstance();
  
  res.set('Content-Type', 'text/plain');
  res.send(await metricsService.getMetrics());
});

export default router;
```

### 4.2 Logging Strategy

#### Structured Logging
```typescript
// src/utils/logger.ts

import winston from 'winston';
import { redactFormat } from 'winston-redact';

const { combine, timestamp, printf, colorize, json } = winston.format;

// Custom log format
const logFormat = printf(({ level, message, timestamp, ...metadata }) => {
  let msg = `${timestamp} [${level}]: ${message}`;
  
  if (Object.keys(metadata).length > 0) {
    msg += ` ${JSON.stringify(metadata)}`;
  }
  
  return msg;
});

// Redaction patterns for sensitive data
const redactPatterns = [
  /password[^"]*"[^"]*"/i,
  /token[^"]*"[^"]*"/i,
  /secret[^"]*"[^"]*"/i,
  /card_number[^"]*"[^"]*"/i,
  /cvv[^"]*"[^"]*"/i,
  /\b\d{16}\b/, // Credit card numbers
  /\b\d{3}-\d{2}-\d{4}\b/, // SSN
];

export const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: combine(
    timestamp({ format: 'YYYY-MM-DD HH:mm:ss.SSS' }),
    redactFormat({ patterns: redactPatterns }),
    process.env.NODE_ENV === 'production' ? json() : logFormat,
    colorize({ enabled: process.env.NODE_ENV !== 'production' })
  ),
  defaultMeta: {
    service: 'ride-hailing-api',
    version: process.env.APP_VERSION || 'unknown',
  },
  transports: [
    // Console transport
    new winston.transports.Console({
      handleExceptions: true,
    }),
    
    // File transport for errors
    new winston.transports.File({
      filename: 'logs/error.log',
      level: 'error',
      maxsize: 5242880, // 5MB
      maxFiles: 5,
    }),
    
    // File transport for all logs
    new winston.transports.File({
      filename: 'logs/combined.log',
      maxsize: 5242880, // 5MB
      maxFiles: 5,
    }),
  ],
  exceptionHandlers: [
    new winston.transports.File({ filename: 'logs/exceptions.log' }),
  ],
  rejectionHandlers: [
    new winston.transports.File({ filename: 'logs/rejections.log' }),
  ],
});

// Request logging middleware
export const requestLogger = (req: any, res: any, next: any) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    
    logger.info('HTTP Request', {
      method: req.method,
      url: req.originalUrl,
      statusCode: res.statusCode,
      duration: `${duration}ms`,
      ip: req.ip,
      userAgent: req.get('user-agent'),
      userId: req.user?.userId,
    });
  });
  
  next();
};

// Error logging middleware
export const errorLogger = (error: any, req: any, res: any, next: any) => {
  logger.error('Unhandled Error', {
    message: error.message,
    stack: error.stack,
    method: req.method,
    url: req.originalUrl,
    body: req.body,
    userId: req.user?.userId,
  });
  
  next(error);
};
```

---

## 5. Disaster Recovery & Backup

### 5.1 Database Backup Strategy

#### Automated Backups
```bash
#!/bin/bash
# scripts/backup-database.sh

set -euo pipefail

BACKUP_BUCKET="ride-hailing-backups"
BACKUP_PREFIX="database/$(date +%Y/%m/%d)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="backup_${TIMESTAMP}.sql.gz"
RETENTION_DAYS=30

# Create backup directory
mkdir -p /tmp/backups

# Dump database
echo "Starting database backup..."
pg_dump \
  --host=$DATABASE_HOST \
  --port=$DATABASE_PORT \
  --username=$DATABASE_USER \
  --dbname=$DATABASE_NAME \
  --format=custom \
  --compress=9 \
  --verbose \
  > /tmp/backups/$BACKUP_FILE

# Upload to S3
echo "Uploading backup to S3..."
aws s3 cp \
  /tmp/backups/$BACKUP_FILE \
  s3://$BACKUP_BUCKET/$BACKUP_PREFIX/$BACKUP_FILE \
  --storage-class STANDARD_IA

# Add metadata
aws s3api put-object-tagging \
  --bucket $BACKUP_BUCKET \
  --key "$BACKUP_PREFIX/$BACKUP_FILE" \
  --tagging "TagSet=[{Key=Environment,Value=$ENVIRONMENT},{Key=Type,Value=DatabaseBackup}]"

# Clean up local backup
rm /tmp/backups/$BACKUP_FILE

# Delete old backups
echo "Cleaning up backups older than $RETENTION_DAYS days..."
aws s3 ls s3://$BACKUP_BUCKET/database/ --recursive | \
  while read -r line; do
    file_date=$(echo $line | awk '{print $1}')
    file_key=$(echo $line | awk '{print $4}')
    
    if [[ $(($(date +%s) - $(date -d "$file_date" +%s))) -gt $((RETENTION_DAYS * 86400)) ]]; then
      aws s3 rm "s3://$BACKUP_BUCKET/$file_key"
      echo "Deleted: $file_key"
    fi
  done

echo "Backup completed successfully!"
```

#### Point-in-Time Recovery
```typescript
// src/admin/restore.service.ts

import { exec } from 'child_process';
import { promisify } from 'util';
import { S3Client, GetObjectCommand } from '@aws-sdk/client-s3';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';

const execAsync = promisify(exec);

export class RestoreService {
  private s3Client: S3Client;
  private backupBucket: string;
  
  constructor() {
    this.s3Client = new S3Client({ region: process.env.AWS_REGION! });
    this.backupBucket = process.env.BACKUP_BUCKET!;
  }
  
  async listBackups(days: number = 30): Promise<string[]> {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - days);
    
    const { Contents } = await this.s3Client.send(/* ListObjectsV2 command */);
    
    return Contents?.filter(obj => {
      const objDate = new Date(obj.LastModified!);
      return objDate >= cutoffDate;
    }).map(obj => obj.Key!) || [];
  }
  
  async restoreFromBackup(backupKey: string): Promise<void> {
    // Download backup from S3
    const tempFile = `/tmp/restore_${Date.now()}.dump`;
    
    const { Body } = await this.s3Client.send(
      new GetObjectCommand({
        Bucket: this.backupBucket,
        Key: backupKey,
      })
    );
    
    const writeStream = createWriteStream(tempFile);
    await pipeline(Body as any, writeStream);
    
    // Restore database
    await execAsync(
      `pg_restore --host=${process.env.DATABASE_HOST} ` +
      `--port=${process.env.DATABASE_PORT} ` +
      `--username=${process.env.DATABASE_USER} ` +
      `--dbname=${process.env.DATABASE_NAME} ` +
      `--clean --if-exists ` +
      tempFile
    );
    
    console.log(`Database restored from ${backupKey}`);
  }
  
  async createPointInTimeRecovery(targetTimestamp: Date): Promise<void> {
    // This would integrate with AWS RDS point-in-time recovery
    // or use WAL archiving for custom implementations
    
    console.log(`Initiating point-in-time recovery to ${targetTimestamp}`);
    
    // Implementation depends on specific database setup
  }
}
```

### 5.2 Multi-Region Failover

#### Failover Configuration
```yaml
# infrastructure/modules/failover/main.tf

resource "aws_route53_health_check" "primary" {
  fqdn              = var.primary_endpoint
  port              = 443
  type              = "HTTPS"
  resource_path     = "/health/ready"
  failure_threshold = 3
  request_interval  = 30
  
  tags = {
    Name = "primary-health-check"
  }
}

resource "aws_route53_health_check" "secondary" {
  fqdn              = var.secondary_endpoint
  port              = 443
  type              = "HTTPS"
  resource_path     = "/health/ready"
  failure_threshold = 3
  request_interval  = 30
  
  tags = {
    Name = "secondary-health-check"
  }
}

resource "aws_route53_record" "failover_primary" {
  zone_id = var.hosted_zone_id
  name    = var.domain_name
  type    = "A"
  
  failover_routing_policy {
    type = "PRIMARY"
  }
  
  set_identifier = "primary"
  
  alias {
    name                   = var.primary_lb_dns_name
    zone_id                = var.primary_lb_zone_id
    evaluate_target_health = true
  }
  
  health_check_id = aws_route53_health_check.primary.id
}

resource "aws_route53_record" "failover_secondary" {
  zone_id = var.hosted_zone_id
  name    = var.domain_name
  type    = "A"
  
  failover_routing_policy {
    type = "SECONDARY"
  }
  
  set_identifier = "secondary"
  
  alias {
    name                   = var.secondary_lb_dns_name
    zone_id                = var.secondary_lb_zone_id
    evaluate_target_health = true
  }
  
  health_check_id = aws_route53_health_check.secondary.id
}
```

---

## 6. Performance Optimization

### 6.1 Caching Strategy

#### Multi-Layer Caching
```typescript
// src/cache/cache.service.ts

import { redisClient } from '../config/redis';
import { LRUCache } from 'lru-cache';

interface CacheOptions {
  ttl?: number;
  prefix?: string;
}

export class CacheService {
  private l1Cache: LRUCache<string, any>;
  private redisPrefix: string;
  
  constructor() {
    // L1: In-memory cache (fastest, limited size)
    this.l1Cache = new LRUCache({
      max: 1000,
      ttl: 60 * 1000, // 1 minute
    });
    
    // L2: Redis cache (distributed, larger capacity)
    this.redisPrefix = process.env.CACHE_PREFIX || 'ride-hailing:';
  }
  
  async get<T>(key: string): Promise<T | null> {
    // Check L1 cache first
    const l1Value = this.l1Cache.get(key);
    if (l1Value !== undefined) {
      return l1Value as T;
    }
    
    // Check L2 cache (Redis)
    const redisKey = `${this.redisPrefix}${key}`;
    const l2Value = await redisClient.get(redisKey);
    
    if (l2Value) {
      const parsed = JSON.parse(l2Value);
      
      // Populate L1 cache
      this.l1Cache.set(key, parsed);
      
      return parsed as T;
    }
    
    return null;
  }
  
  async set(key: string, value: any, options: CacheOptions = {}): Promise<void> {
    const { ttl = 3600, prefix = '' } = options;
    
    // Set in L1 cache
    this.l1Cache.set(key, value);
    
    // Set in L2 cache
    const redisKey = `${this.redisPrefix}${prefix}${key}`;
    await redisClient.setEx(redisKey, ttl, JSON.stringify(value));
  }
  
  async delete(key: string, prefix: string = ''): Promise<void> {
    // Delete from L1 cache
    this.l1Cache.delete(key);
    
    // Delete from L2 cache
    const redisKey = `${this.redisPrefix}${prefix}${key}`;
    await redisClient.del(redisKey);
  }
  
  async invalidatePattern(pattern: string): Promise<void> {
    const redisPattern = `${this.redisPrefix}${pattern}`;
    const keys = await redisClient.keys(redisPattern);
    
    if (keys.length > 0) {
      await redisClient.del(keys);
    }
    
    // Clear L1 cache entries matching pattern
    for (const key of this.l1Cache.keys()) {
      if (key.match(pattern)) {
        this.l1Cache.delete(key);
      }
    }
  }
  
  // Specialized cache methods for ride-hailing
  async getCachedDriverLocation(driverId: string): Promise<{
    latitude: number;
    longitude: number;
    timestamp: number;
  } | null> {
    return this.get(`driver:location:${driverId}`);
  }
  
  async setDriverLocation(
    driverId: string,
    location: { latitude: number; longitude: number }
  ): Promise<void> {
    await this.set(`driver:location:${driverId}`, {
      ...location,
      timestamp: Date.now(),
    }, { ttl: 30 }); // 30 seconds TTL for real-time data
  }
  
  async getCachedTrip(tripId: string): Promise<any | null> {
    return this.get(`trip:${tripId}`);
  }
  
  async setTrip(tripId: string, tripData: any): Promise<void> {
    await this.set(`trip:${tripId}`, tripData, { ttl: 86400 }); // 24 hours
  }
  
  async getCachedFareEstimate(
    pickupLat: number,
    pickupLng: number,
    dropoffLat: number,
    dropoffLng: number,
    vehicleType: string
  ): Promise<number | null> {
    const key = `fare:${pickupLat}:${pickupLng}:${dropoffLat}:${dropoffLng}:${vehicleType}`;
    return this.get(key);
  }
  
  async setFareEstimate(
    pickupLat: number,
    pickupLng: number,
    dropoffLat: number,
    dropoffLng: number,
    vehicleType: string,
    fare: number
  ): Promise<void> {
    const key = `fare:${pickupLat}:${pickupLng}:${dropoffLat}:${dropoffLng}:${vehicleType}`;
    await this.set(key, fare, { ttl: 300 }); // 5 minutes
  }
}
```

### 6.2 Database Query Optimization

#### Index Strategy
```sql
-- migrations/create_indexes.sql

-- Users table
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);
CREATE INDEX CONCURRENTLY idx_users_phone ON users(phone_number);
CREATE INDEX CONCURRENTLY idx_users_created_at ON users(created_at);

-- Drivers table
CREATE INDEX CONCURRENTLY idx_drivers_user_id ON drivers(user_id);
CREATE INDEX CONCURRENTLY idx_drivers_status ON drivers(status);
CREATE INDEX CONCURRENTLY idx_drivers_current_location ON drivers USING GIST (current_location);
CREATE INDEX CONCURRENTLY idx_drivers_rating ON drivers(rating DESC);

-- Trips table
CREATE INDEX CONCURRENTLY idx_trips_rider_id ON trips(rider_id);
CREATE INDEX CONCURRENTLY idx_trips_driver_id ON trips(driver_id);
CREATE INDEX CONCURRENTLY idx_trips_status ON trips(status);
CREATE INDEX CONCURRENTLY idx_trips_created_at ON trips(created_at DESC);
CREATE INDEX CONCURRENTLY idx_trips_pickup_location ON trips USING GIST (pickup_location);
CREATE INDEX CONCURRENTLY idx_trips_dropoff_location ON trips USING GIST (dropoff_location);
CREATE INDEX CONCURRENTLY idx_trips_status_created ON trips(status, created_at DESC);

-- Payments table
CREATE INDEX CONCURRENTLY idx_payments_trip_id ON payments(trip_id);
CREATE INDEX CONCURRENTLY idx_payments_user_id ON payments(user_id);
CREATE INDEX CONCURRENTLY idx_payments_status ON payments(status);
CREATE INDEX CONCURRENTLY idx_payments_created_at ON payments(created_at DESC);

-- Driver locations (for real-time tracking)
CREATE INDEX CONCURRENTLY idx_driver_locations_driver_id ON driver_locations(driver_id);
CREATE INDEX CONCURRENTLY idx_driver_locations_timestamp ON driver_locations(timestamp DESC);
CREATE INDEX CONCURRENTLY idx_driver_locations_composite ON driver_locations(driver_id, timestamp DESC);

-- Composite indexes for common queries
CREATE INDEX CONCURRENTLY idx_trips_rider_status ON trips(rider_id, status);
CREATE INDEX CONCURRENTLY idx_trips_driver_status ON trips(driver_id, status);
```

---

## 7. Environment Configuration

### 7.1 Environment Variables

```bash
# .env.example

# Application
NODE_ENV=production
APP_VERSION=1.0.0
PORT=3000
API_PREFIX=/api/v1

# Database
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=ride_hailing
DATABASE_USER=postgres
DATABASE_PASSWORD=secure_password_here
DATABASE_POOL_MIN=2
DATABASE_POOL_MAX=20
DATABASE_SSL=true

# Redis
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=secure_redis_password
REDIS_DB=0
REDIS_TLS_ENABLED=true

# JWT
JWT_ACCESS_SECRET=your_access_secret_here
JWT_REFRESH_SECRET=your_refresh_secret_here
JWT_ACCESS_EXPIRY=15m
JWT_REFRESH_EXPIRY=7d

# Data Encryption
DATA_ENCRYPTION_KEY=your_32_char_encryption_key

# AWS
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
S3_BUCKET=ride-hailing-media
CLOUDFRONT_DOMAIN=cdn.ridehailing.com

# Stripe
STRIPE_SECRET_KEY=sk_live_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx
STRIPE_CONNECT_CLIENT_ID=ca_xxx

# Twilio
TWILIO_ACCOUNT_SID=ACxxx
TWILIO_AUTH_TOKEN=your_auth_token
TWILIO_PHONE_NUMBER=+1234567890

# Firebase
FIREBASE_PROJECT_ID=your-project-id
FIREBASE_PRIVATE_KEY=-----BEGIN PRIVATE KEY-----\n...
FIREBASE_CLIENT_EMAIL=firebase-adminsdk-xxx@your-project.iam.gserviceaccount.com

# Maps
GOOGLE_MAPS_API_KEY=your_api_key
MAPBOX_ACCESS_TOKEN=your_token

# Monitoring
SENTRY_DSN=https://xxx@sentry.io/xxx
DATADOG_API_KEY=your_api_key
LOG_LEVEL=info

# Security
ALLOWED_ORIGINS=https://app.ridehailing.com,https://admin.ridehailing.com
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=100

# Backup
BACKUP_BUCKET=ride-hailing-backups
BACKUP_RETENTION_DAYS=30
```

### 7.2 Docker Configuration

```dockerfile
# Dockerfile

FROM node:20-alpine AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

FROM node:20-alpine AS production

ARG NODE_ENV=production
ENV NODE_ENV=${NODE_ENV}

RUN addgroup -g 1001 -S nodejs && \
    adduser -S nodejs -u 1001

WORKDIR /app

COPY --from=builder --chown=nodejs:nodejs /app/node_modules ./node_modules
COPY --from=builder --chown=nodejs:nodejs /app/dist ./dist
COPY --from=builder --chown=nodejs:nodejs /app/package.json ./

USER nodejs

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health/live || exit 1

CMD ["node", "dist/server.js"]
```

```yaml
# docker-compose.yml

version: '3.8'

services:
  api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=development
      - DATABASE_URL=postgresql://postgres:postgres@db:5432/ride_hailing
      - REDIS_URL=redis://redis:6379
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    volumes:
      - ./logs:/app/logs
    networks:
      - ride-hailing-network

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: ride_hailing
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - ride-hailing-network

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - ride-hailing-network

volumes:
  postgres_data:
  redis_data:

networks:
  ride-hailing-network:
    driver: bridge
```

---

## 8. Security Best Practices Checklist

### Infrastructure Security
- [ ] VPC with private subnets for databases and caches
- [ ] Security groups with minimal required ports
- [ ] Network ACLs for additional layer of protection
- [ ] WAF rules for common web vulnerabilities
- [ ] DDoS protection via AWS Shield
- [ ] Encrypted communication (TLS 1.3) everywhere
- [ ] Secrets management via AWS Secrets Manager
- [ ] Regular security patching automation

### Application Security
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention via parameterized queries
- [ ] XSS protection via Content Security Policy
- [ ] CSRF protection for state-changing operations
- [ ] Rate limiting on all public endpoints
- [ ] Authentication required for sensitive operations
- [ ] Authorization checks on every request
- [ ] Audit logging for security events

### Data Security
- [ ] Encryption at rest (AES-256)
- [ ] Encryption in transit (TLS)
- [ ] Field-level encryption for PII
- [ ] Secure key rotation procedures
- [ ] Data masking in logs
- [ ] GDPR compliance measures
- [ ] Data retention policies
- [ ] Right to deletion implementation

### Operational Security
- [ ] Least privilege IAM policies
- [ ] Multi-factor authentication for admin access
- [ ] Regular security audits
- [ ] Penetration testing schedule
- [ ] Incident response plan
- [ ] Security training for team
- [ ] Vulnerability disclosure program
- [ ] Compliance certifications (SOC 2, ISO 27001)

---

## 9. Deployment Checklist

### Pre-Deployment
- [ ] All tests passing (unit, integration, e2e)
- [ ] Security scan completed with no critical issues
- [ ] Code review approved
- [ ] Database migrations tested on staging
- [ ] Environment variables configured
- [ ] SSL certificates valid
- [ ] Monitoring dashboards configured
- [ ] Alert thresholds set
- [ ] Rollback plan documented

### Deployment
- [ ] Deploy to staging environment
- [ ] Run smoke tests on staging
- [ ] Stakeholder approval obtained
- [ ] Deploy to production (blue-green)
- [ ] Monitor error rates and latency
- [ ] Verify all health checks passing
- [ ] Confirm metrics flowing correctly

### Post-Deployment
- [ ] Run post-deployment verification tests
- [ ] Monitor for 30 minutes
- [ ] Update documentation
- [ ] Notify stakeholders
- [ ] Schedule retrospective if issues occurred

---

## 10. Incident Response

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| P0 | Critical - System down | < 15 min | Complete outage, data breach |
| P1 | High - Major feature broken | < 1 hour | Payments failing, trips not matching |
| P2 | Medium - Partial degradation | < 4 hours | Slow performance, non-critical bugs |
| P3 | Low - Minor issues | < 24 hours | UI glitches, cosmetic issues |

### Escalation Path
1. On-call engineer (PagerDuty)
2. Engineering lead
3. VP of Engineering
4. CTO

### Communication Templates
- Status page updates
- Customer communications
- Internal stakeholder updates
- Post-mortem documentation

---

## Conclusion

Phase 7 establishes the critical infrastructure, security, and operational foundations for the ride-hailing platform. This phase ensures:

✅ **Scalable Infrastructure**: Auto-scaling, load balancing, multi-AZ deployment
✅ **Robust Security**: Encryption, authentication, authorization, compliance
✅ **Reliable Operations**: Monitoring, alerting, backup, disaster recovery
✅ **Efficient Deployment**: CI/CD pipelines, blue-green deployments
✅ **Performance Optimization**: Caching, database optimization, CDN

This foundation enables the platform to handle production traffic securely and reliably while maintaining high availability and performance standards.
