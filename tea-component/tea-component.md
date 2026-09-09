# The TEA Component object

The TEA Component represents a component lineage. A product release may
be constructed with one or multiple TEA Components, each with their own set of
releases and related artefacts. The component has releases, which has a set
of artifacts for each release. One artifact may belong to multiple releases.

Each TEA Component has a list of Component Releases (see `/component/{uuid}/releases`),
which enumerates all known versions for that component.

The TEA API is agnostic as to how a "version" is indicated - semver, vers,
name, hash or anything else.

## Versions and TEIs

Each product release and product object has one or multiple TEI URNs.

For the API to be able to present a list of versions in a cronological order,
a timestamp for a release is required.

## TEA Component Object

A TEA Component object has the following parts:

- __uuid__: A unique identifier for the TEA component
- __name__: Component name
- __identifiers__: List of identifiers for the component
  - __idType__: Type of identifier, e.g. `TEI`, `PURL`, `CPE`
  - __idValue__: Identifier value

Required fields:

- uuid, name, identifiers

### Examples

Some examples of Maven artefacts as TEA Components:

```json
{
  "uuid": "3910e0fd-aff4-48d6-b75f-8bf6b84687f0",
  "name": "Apache Log4j API",
  "identifiers": [
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-api"
    }
  ]
}
```

```json
{
  "uuid": "b844c9bd-55d6-478c-af59-954a932b6ad3",
  "name": "Apache Log4j Core",
  "identifiers": [
    {
      "idType": "CPE",
      "idValue": "cpe:2.3:a:apache:log4j"
    },
    {
      "idType": "PURL",
      "idValue": "pkg:maven/org.apache.logging.log4j/log4j-core"
    }
  ]
}
```

## References

- Semantic versioning (Semver): <https://semver.org>
- PURL VERS <https://github.com/package-url/purl-spec/blob/master/VERSION-RANGE-SPEC.rst>
