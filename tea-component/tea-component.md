# The TEA Component API

The TEA Component object is the object that indicates a product component. The product may
be constructed with one or multiple Tea Components, each with their own set of
related artefacts.

For each TEA COMPONENT there is a TEA COMPONENT INDEX, which is a list of all versions
for that component.

The API should be very agnostic as to how a "version" is indicated - semver, vers,
name, hash or anything else.

## Major and minor versions

Each component is for a sub-version or minor version (using semver definitions). A new
major version counts as a new product with a separate product object (TPO). Each
product object has one or multiple TEI URNs.

For the API to be able to present a list of versions in a cronological order,
a timestamp for a release is required.

## TEA Component Object

A TEA Component object has the following parts:

- __uuid__: A unique identifier for the TEA component
- __name__: Component name
- __identifiers__: List of identifiers for the component
  - __idType__: Type of identifier, e.g. `tei`, `purl`, `cpe`
  - __idValue__: Identifier value

Note: In coming versions, there may be a flag indicating lifecycle status
for a component.

## Handling the Pre-Release flag

The "Pre-release" flag is used to indicate that this is not a final release.
For a given Component with a UUID, the flag can be set to indicate a "test", "beta", "alpha"
or similar non-deployed release. It can only be set when creating the Component.
The TEA implementation may allow it to be unset (False) once. This is to support
situations where a object is promoted as is after testing to production version. The flag can not
be set after initial creation and publication of the Component.

If the final version is different from the pre-release (bugs fixed, code changed, different binary)
a new Component with a new UUID and version needs to be created.

## References

- Semantic versioning (Semver): <https://semver.org>
- PURL VERS <https://github.com/package-url/purl-spec/blob/master/VERSION-RANGE-SPEC.rst>
