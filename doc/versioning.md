# Versioning

The Transparency Exchange API (TEA) uses semantic versioning for API evolution. This document describes the versioning strategy, compatibility guarantees, and migration guidelines.

## Semantic Versioning

TEA follows [Semantic Versioning 2.0.0](https://semver.org/) for API versions:

```
MAJOR.MINOR.PATCH
```

### Version Components

- **MAJOR**: Breaking changes that require client updates
- **MINOR**: Backward-compatible additions (new endpoints, optional fields)
- **PATCH**: Backward-compatible bug fixes

### Pre-release Versions

Pre-release versions use the format:

```
MAJOR.MINOR.PATCH-PRERELEASE
```

Examples:

- `1.0.0-alpha.1`
- `1.0.0-beta.2`
- `1.0.0-rc.1`

## API Versioning Strategy

### URL Versioning

API versions are included in the URL path:

```
/v{MAJOR}/...
```

Current version: `v1`

### Content Negotiation

For content that may evolve independently of the API version, use content negotiation with the `Accept` header:

```
Accept: application/vnd.cyclonedx+json; version=1.5
```

## Compatibility Guarantees

### Backward Compatibility

- **PATCH** versions: Fully backward compatible
- **MINOR** versions: Backward compatible additions only
- **MAJOR** versions: May include breaking changes

### Forward Compatibility

Clients SHOULD ignore unknown fields in responses. Servers MUST NOT require unknown fields in requests.

### Deprecation Policy

1. Features are marked as deprecated in MINOR releases
2. Deprecated features are removed in the next MAJOR release
3. Deprecation notices include:
   - Deprecation version
   - Removal version
   - Migration guidance

## Version Discovery

### Well-Known Endpoint

Clients discover available API versions through `/.well-known/tea`:

```json
{
  "schemaVersion": 1,
  "endpoints": [
    {
      "url": "https://api.example.com/tea/v1",
      "versions": ["1.0.0", "1.1.0"],
      "priority": 1
    }
  ]
}
```

### Version Headers

Servers MAY include version information in responses:

```
X-API-Version: 1.0.0
```

## Migration Guidelines

### Minor Version Upgrades

1. Review release notes for new features
2. Update client code to handle new optional fields
3. Test with new version in staging environment
4. Gradually roll out updated clients

### Major Version Upgrades

1. Review breaking changes documentation
2. Update client code for required changes
3. Implement feature flags if needed
4. Test extensively in staging
5. Plan rollback strategy
6. Execute blue-green deployment

### Testing Strategy

- Maintain test suites for multiple API versions during transition periods
- Use contract testing to validate compatibility
- Implement canary deployments for gradual rollout

## Implementation Considerations

### Server-Side

- Support multiple concurrent API versions
- Use version-aware routing
- Implement graceful degradation for older clients
- Provide version-specific documentation

### Client-Side

- Implement version negotiation logic
- Handle version-specific response formats
- Provide upgrade prompts for deprecated versions
- Support fallback to older versions when possible

## Version Support Policy

- Current MAJOR version receives active development and support
- Previous MAJOR version receives security updates only
- Versions older than N-1 MAJOR releases are deprecated
- Deprecation notices provided 6 months before end of support
