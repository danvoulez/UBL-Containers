# ✅ Railway Deployment Checklist

This checklist will help you deploy UBL Server to Railway successfully.

## Pre-Deployment Checklist

### 1. Repository Setup
- [ ] All changes committed and pushed to GitHub
- [ ] Dockerfile is present in the repository root
- [ ] railway.toml is configured
- [ ] .dockerignore is present for optimized builds

### 2. Railway Account Setup
- [ ] Railway account created at https://railway.app
- [ ] GitHub account connected to Railway
- [ ] Billing information added (if needed beyond free tier)

## Deployment Steps

### 3. Create Railway Project
- [ ] Create new project on Railway
- [ ] Connect GitHub repository (UBL-Containers)
- [ ] Select correct branch

### 4. Add PostgreSQL Database
- [ ] Add PostgreSQL database service to project
- [ ] Wait for PostgreSQL to provision
- [ ] Verify DATABASE_URL environment variable is auto-set

### 5. Initialize Database Schema
Choose one method:

#### Option A: Railway CLI (Recommended)
```bash
# Install CLI
npm i -g @railway/cli

# Login and link
railway login
railway link

# Run migrations
railway run psql $DATABASE_URL -f sql/000_unified.sql
```

#### Option B: Manual
- [ ] Copy DATABASE_URL from Railway dashboard
- [ ] Connect using PostgreSQL client (psql, DBeaver, etc.)
- [ ] Execute sql/000_unified.sql

### 6. Configure Environment Variables
Set these in Railway dashboard under your service → Variables:

- [ ] `WEBAUTHN_RP_ID` = your-app.railway.app (your actual Railway domain)
- [ ] `WEBAUTHN_ORIGIN` = https://your-app.railway.app (full URL with https://)
- [ ] `PORT` (auto-set by Railway, verify it exists)
- [ ] `DATABASE_URL` (auto-set by PostgreSQL service, verify it exists)

### 7. Deploy Application
- [ ] Trigger deployment (automatic on git push or manual via dashboard)
- [ ] Monitor build logs for errors
- [ ] Wait for deployment to complete

### 8. Verify Deployment
- [ ] Check health endpoint: `curl https://your-app.railway.app/health`
- [ ] Expected response: `{"status":"healthy","version":"2.0.0+postgres"}`
- [ ] Verify application logs show no errors
- [ ] Test a basic API endpoint

## Post-Deployment Verification

### 9. Functional Testing
- [ ] Test /state/:container_id endpoint
- [ ] Test /link/validate endpoint
- [ ] Test /link/commit endpoint
- [ ] Test WebAuthn registration (if using frontend)
- [ ] Test WebAuthn login (if using frontend)

### 10. Monitoring Setup
- [ ] Check Railway metrics dashboard
- [ ] Verify health checks are working
- [ ] Set up alerts (if needed)
- [ ] Monitor resource usage

### 11. Security Verification
- [ ] Verify HTTPS is working (Railway provides automatic SSL)
- [ ] Check that sensitive environment variables are not exposed
- [ ] Verify WebAuthn is working with HTTPS
- [ ] Test rate limiting on ID endpoints

## Troubleshooting

If deployment fails, check:
- [ ] Build logs in Railway dashboard
- [ ] Environment variables are set correctly
- [ ] Database is initialized with schema
- [ ] PostgreSQL service is running
- [ ] Dockerfile syntax is correct

## Optional: Custom Domain

If using a custom domain:
- [ ] Add custom domain in Railway settings
- [ ] Update DNS records (CNAME or A record)
- [ ] Wait for DNS propagation (up to 24 hours)
- [ ] Update WEBAUTHN_RP_ID to custom domain
- [ ] Update WEBAUTHN_ORIGIN to use custom domain
- [ ] Verify SSL certificate is issued for custom domain

## Resources

- **Full Guide:** [RAILWAY.md](RAILWAY.md)
- **Railway Docs:** https://docs.railway.app
- **PostgreSQL Guide:** https://docs.railway.app/databases/postgresql
- **Support:** https://discord.gg/railway

## Success Criteria

Your deployment is successful when:
- ✅ Health check returns 200 OK with correct JSON
- ✅ PostgreSQL connection is working
- ✅ API endpoints respond correctly
- ✅ WebAuthn functionality works over HTTPS
- ✅ No errors in application logs
- ✅ Resource usage is within expected limits

---

**Need help?** Check [RAILWAY.md](RAILWAY.md) for detailed troubleshooting steps.
