# Deploying to Vercel

## Quick Setup

When importing this project to Vercel, use these settings:

### Project Settings

1. **Framework Preset**: Docusaurus (v2+) *(should auto-detect)*
2. **Root Directory**: `docs`
3. **Build Command**: `yarn build` or `docusaurus build` *(default)*
4. **Output Directory**: `build` *(default)*
5. **Install Command**: `yarn install` *(default)*

### Environment Variables

No environment variables are required for the documentation site to build.

## Deployment Steps

### Option 1: Auto-detected (Recommended)

1. Import project from GitHub
2. Set **Root Directory** to `docs`
3. Vercel should auto-detect Docusaurus and set everything else correctly
4. Click **Deploy**

### Option 2: Manual Configuration

If auto-detection doesn't work, manually configure:

```
Framework Preset: Other
Root Directory: docs
Build Command: yarn build
Output Directory: build
Install Command: yarn install
```

## Vercel Configuration File

Alternatively, the project includes a `vercel.json` at the root that specifies the build configuration. Vercel will automatically use this file.

## Troubleshooting

### Build fails with "Cannot find module"
- Make sure **Root Directory** is set to `docs`
- Verify all dependencies are listed in `docs/package.json`

### Missing documents error
- This should be fixed with the restructure
- All markdown files are now in `docs/docs/`
- `sidebars.ts` references match actual file names

### Broken links warning
- These are warnings and won't break the deployment
- Configuration uses `onBrokenLinks: 'warn'` which allows build to succeed
- Fix broken links by checking the build output and updating markdown files

## Local Testing Before Deploy

Always test the build locally before deploying:

```bash
cd docs
yarn install
yarn build
yarn serve  # Preview the production build
```

## Post-Deployment

After successful deployment:

1. Check that all pages load correctly
2. Verify navigation works
3. Test internal links
4. Ensure images and assets load properly

## Custom Domain (Optional)

To add a custom domain:

1. Go to Project Settings in Vercel
2. Navigate to Domains
3. Add your domain and follow DNS configuration instructions

