# 🚂 Railway Deployment Guide

This guide explains how to deploy UBL Server to [Railway](https://railway.app).

## Prerequisites

- A Railway account ([sign up here](https://railway.app))
- GitHub repository connected to Railway
- PostgreSQL database service on Railway

## Quick Deploy

### 1. Create a New Project on Railway

1. Go to [Railway](https://railway.app)
2. Click "New Project"
3. Select "Deploy from GitHub repo"
4. Choose your `UBL-Containers` repository

### 2. Add PostgreSQL Database

1. In your Railway project, click "+ New"
2. Select "Database" → "Add PostgreSQL"
3. Railway will automatically provision a PostgreSQL database
4. The `DATABASE_URL` environment variable will be automatically set

### 3. Initialize the Database

Once PostgreSQL is running, you need to initialize the database schema:

#### Option A: Using Railway CLI (Recommended)

```bash
# Install Railway CLI
npm i -g @railway/cli

# Login to Railway
railway login

# Link to your project
railway link

# Connect to PostgreSQL and run migrations
railway run psql $DATABASE_URL -f sql/000_unified.sql
```

#### Option B: Manual via Railway Dashboard

1. Go to your PostgreSQL service in Railway
2. Click "Connect" → "Postgres Connection URL"
3. Copy the connection URL
4. Use a PostgreSQL client to connect and run:
   ```sql
   \i /path/to/sql/000_unified.sql
   ```

### 4. Configure Environment Variables

Railway will automatically set `DATABASE_URL` and `PORT`. You need to add:

1. In your Railway project, go to your service
2. Click "Variables" tab
3. Add the following variables:

| Variable | Value | Description |
|----------|-------|-------------|
| `WEBAUTHN_RP_ID` | `your-app.railway.app` | Your Railway domain |
| `WEBAUTHN_ORIGIN` | `https://your-app.railway.app` | Full URL with https:// |

**Note:** Replace `your-app` with your actual Railway-generated domain or custom domain.

### 5. Deploy

1. Railway will automatically deploy when you push to your connected branch
2. Or click "Deploy" in the Railway dashboard
3. Monitor the build logs in the Railway dashboard

### 6. Verify Deployment

Once deployed, test your deployment:

```bash
# Health check
curl https://your-app.railway.app/health

# Expected response:
# {"status":"healthy","version":"2.0.0+postgres"}
```

## Configuration Files

The following files configure Railway deployment:

- **`Dockerfile`** - Multi-stage Docker build for optimized images
- **`railway.toml`** - Railway-specific configuration
- **`.env.example`** - Template for environment variables

## Environment Variables Reference

### Required Variables

| Variable | Example | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://...` | PostgreSQL connection (auto-set by Railway) |
| `PORT` | `8080` | Server port (auto-set by Railway) |
| `WEBAUTHN_RP_ID` | `your-app.railway.app` | WebAuthn relying party ID |
| `WEBAUTHN_ORIGIN` | `https://your-app.railway.app` | WebAuthn origin URL |

### Optional Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (trace, debug, info, warn, error) |

## Custom Domains

To use a custom domain:

1. Go to your service in Railway
2. Click "Settings" → "Domains"
3. Click "Add Domain"
4. Follow Railway's instructions for DNS configuration
5. Update `WEBAUTHN_RP_ID` and `WEBAUTHN_ORIGIN` to use your custom domain

## Database Migrations

When deploying updates that require database schema changes:

1. Add new SQL migration files in the `sql/` directory
2. Run migrations using Railway CLI:
   ```bash
   railway run psql $DATABASE_URL -f sql/001_your_migration.sql
   ```

## Monitoring

### Health Checks

Railway automatically monitors the `/health` endpoint:
- **Healthy:** Returns `{"status":"healthy","version":"2.0.0+postgres"}`
- **Unhealthy:** Service will automatically restart

### Logs

View logs in Railway dashboard:
1. Go to your service
2. Click "Logs" tab
3. Filter by severity level if needed

### Metrics

Access Prometheus metrics at `/metrics` endpoint:
```bash
curl https://your-app.railway.app/metrics
```

## Troubleshooting

### Build Fails

1. Check build logs in Railway dashboard
2. Ensure all dependencies are in `Cargo.toml`
3. Verify Dockerfile syntax

### Database Connection Errors

1. Verify PostgreSQL service is running
2. Check `DATABASE_URL` is set correctly
3. Ensure database schema is initialized
4. Check PostgreSQL logs in Railway

### WebAuthn Errors

1. Verify `WEBAUTHN_RP_ID` matches your domain (without protocol)
2. Verify `WEBAUTHN_ORIGIN` includes `https://` protocol
3. Ensure domain is accessible via HTTPS

### Service Won't Start

1. Check application logs for errors
2. Verify all environment variables are set
3. Test health endpoint: `curl https://your-app.railway.app/health`
4. Check if PORT binding is correct (Railway auto-sets this)

## Scaling

Railway supports both vertical and horizontal scaling:

### Vertical Scaling
1. Go to service "Settings"
2. Adjust "Resources" (CPU/RAM)

### Horizontal Scaling
Due to PostgreSQL SERIALIZABLE transactions, consider:
- Read replicas for read-heavy workloads
- Connection pooling (built into sqlx)

## Cost Optimization

- Railway charges based on usage
- Starter plan includes $5 free credit/month
- Monitor usage in Railway dashboard
- Optimize Docker image size (currently using multi-stage build)

## Security Best Practices

1. **Environment Variables:** Never commit `.env` files
2. **Database:** Use strong passwords (Railway auto-generates)
3. **HTTPS:** Railway provides automatic HTTPS
4. **Updates:** Keep dependencies updated regularly
5. **Rate Limiting:** Built-in rate limiting for WebAuthn endpoints

## Support

- **Railway Docs:** https://docs.railway.app
- **UBL Issues:** https://github.com/danvoulez/UBL-Containers/issues
- **Railway Discord:** https://discord.gg/railway

## Additional Resources

- [Railway Documentation](https://docs.railway.app)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [PostgreSQL on Railway](https://docs.railway.app/databases/postgresql)
- [UBL Architecture](./ARCHITECTURE.md)
