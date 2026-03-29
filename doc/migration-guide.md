# Migration Guide

This guide provides instructions for migrating between versions of the Transparency Exchange API (TEA). Follow these steps carefully to ensure compatibility and minimize downtime.

## Version Compatibility Matrix

| From Version | To Version | Migration Path          | Breaking Changes      |
| ------------ | ---------- | ----------------------- | --------------------- |
| 0.1.x        | 0.2.x      | Direct                  | None                  |
| 0.2.x        | 0.3.x      | Direct                  | None                  |
| 0.3.x        | 1.0.0      | Direct                  | None (beta to stable) |
| 1.0.x        | 1.1.x      | Direct                  | None                  |
| 1.x.x        | 2.0.0      | Review breaking changes | Yes                   |

## General Migration Steps

### 1. Review Release Notes

Before migrating, carefully review the release notes for the target version:

- New features and endpoints
- Deprecated functionality
- Required configuration changes
- Known issues and workarounds

### 2. Update Dependencies

Update TEA client libraries and dependencies to compatible versions:

```bash
# Example for different languages
npm update @cyclonedx/tea-client
pip install --upgrade cyclonedx-tea
cargo update cyclonedx-tea
```

### 3. Update Configuration

Review and update configuration files:

```yaml
# config.yaml
api:
  version: "1.0.0"  # Update to target version
  baseUrl: "https://api.example.com/tea/v1"
```

### 4. Update Code

Modify client code for any API changes:

```javascript
// Before (v0.3.x)
const client = new TeaClient({
  baseUrl: 'https://api.example.com/tea'
});

// After (v1.0.0)
const client = new TeaClient({
  baseUrl: 'https://api.example.com/tea/v1',
  version: '1.0.0'
});
```

### 5. Test Migration

Test the migration in a staging environment:

```bash
# Run integration tests
npm test
pytest tests/
cargo test
```

### 6. Deploy Gradually

Use canary deployments or feature flags:

```bash
# Deploy to 10% of traffic first
kubectl set image deployment/tea-client tea-client:v1.0.0
kubectl rollout status deployment/tea-client
```

## Version-Specific Migrations

### Migrating from 0.3.x to 1.0.0

#### API Changes

- No breaking changes - this is a stability release
- All beta features are now stable
- Improved error messages and validation

#### Client Changes

```typescript
// Update version specification
const client = new TeaClient({
  version: '1.0.0'  // Explicitly specify version
});
```

#### Server Changes

- Update server configuration for stable endpoints
- Review authentication settings
- Update monitoring and logging

### Migrating from 1.0.x to 1.1.x

#### New Features

- Enhanced insights API
- Improved pagination support
- Additional artifact types

#### Backward Compatibility

- All existing clients continue to work
- New features are opt-in

#### Optional Updates

```python
# Use new insights features
insights = client.insights.query("component.name == 'openssl'")
```

## Troubleshooting

### Common Issues

#### Authentication Failures

- Verify token scopes are correct for v1 endpoints
- Check mTLS certificate validity
- Confirm issuer configuration

#### Version Negotiation

- Ensure client specifies correct API version
- Check server version support
- Review well-known endpoint configuration

#### Performance Issues

- Monitor rate limits after migration
- Check for N+1 query problems
- Optimize batch operations

### Rollback Plan

Always prepare a rollback strategy:

```bash
# Quick rollback command
kubectl rollout undo deployment/tea-client
```

## Support

For migration assistance:

- Review the [TEA documentation](https://github.com/CycloneDX/transparency-exchange-api)
- Check the [GitHub issues](https://github.com/CycloneDX/transparency-exchange-api/issues) for known issues
- Contact the TEA working group for complex migrations

## Post-Migration Validation

After migration, validate:

- [ ] All existing functionality works
- [ ] New features are accessible
- [ ] Performance meets requirements
- [ ] Monitoring and alerting are configured
- [ ] Documentation is updated
