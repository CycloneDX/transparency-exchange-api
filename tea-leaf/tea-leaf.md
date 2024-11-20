# The TEA Leaf Object (TLO)

_Work in progress_

The TEA LEAF is the object that indicates a product version. The API should be
very agnostic as to how a "version" is indicated - semver, vers, name, hash or anything else.

## Major and minor versions

Each leaf is for a sub-version or minor version (using semver definitions). A new
major version counts as a new product with a separate product object (TPO). Each
product object has one or multiple TEI URNs.

For the API to be able to present a list of versions in a cronological order,
a timestamp for a release is required.

## The Leaf Object

- __Uuid__ unique for this object
- __Product name__: A text field
- __Product version__: A text field, no required syntax
- __Release date__: A unix timestamp
- __Pre-Release__: A boolean flag indicating a pre-release (beta, rc)
- __Tco_uuid__: A reference to the TEA Collection objet for this release

## Handling the Pre-Release flag

The Prerelease flag is used to indicate that this is not a final release. For a given Leaf with a UUID, the flag
can be set to indicate a "test", "beta", "alfa" or similar non-deployed release. It can only be set when
creating the LEAF. The TEA implementation may allow it to be unset (False) once. This is to support
situations where a object is promoted as is after testing to production version. The flag can not
be set after initial creation and publication of the leaf.

If the final version is different from the pre-release (bugs fixed, code changed, different binary)
a new leaf with a new UUID and version needs to be created.

## References

- Semantic versioning (Semver): <https://semver.org>