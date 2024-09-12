# The TEA Leaf Object (TLO)

_Work in progress_

The TEA LEAF is the object that indicates a product version. The API should be
very agnostic to how a "version" is indicated - semver, name, hash or anything else.

## Major and minor versions

Each leaf is for a sub-version or minor version (using semver definitions). A new
major version counts as a new product with a separate product object (TPO). Each
product object has one or multiple TEI URNs.

For the API to be able to present a list of versions in a cronological order,
a timestamp for a release is required.

## The Leaf Object

- __UUID__ unique for this object
- __Product name__: A text field
- __Product version__: A text field, no required syntax
- __Release date__: A unix timestamp
- __Prerelease__: A flag indicating a pre-release (beta, rc)
- __TCO_UUID__: A reference to the TEA Collection objet for this release

## References

- Semantic versioning (Semver): <https://semver.org>