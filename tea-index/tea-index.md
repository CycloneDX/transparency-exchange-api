# The TEA index API

After TEA discovery, a user has a URL to the TEA index which will provide
an entrypoint in the Transparency Exchange API where a list of all
versions of a product can be found

## Authorization

Authorization can be done on multiple levels, including
which versions is supported for a specific user.

## Composite products

If a product consists of a set of products, each with a different
version number and update scheme, a TEA bundle will be the starting
point of discovery. The TEA bundle will list all included parts
and include pointers (URLs) to the TEA index for these.

The URL can be to a different vendor or different site with the
same vendor.

## TEA index structure

The goal of the TEA index is to provide a selection of product
versions to assist the user software in finding a match for the
owned version.

### API usage

The user will find this API end point using TEA discovery.

A user will approach the API just to discover data before purchase,
or with a specific product and product version in scope.
The format of the version may follow many syntaxes, so maybe
the API needs to be able to provide some sort of format
for the version string.

An automated system may want to provide the user with a GUI,
listing versions and being able to scroll to the next page
until the user selects a version.

### Content of the index object

* UUID
* Identifier (name)
* Version tag
* Metadata
  * CLE, Common lifecycle enumeration
    * Note: This product version is no longer supported
    * Status: Beta, prod, deprecate?
    * Release date (in the CLE)
* Key-value tags
  * Defined in API
  * TBD: Do we allow Vendor extensions?

### Tea API operation - index files

* Recommendation: Support HTTP compression
* Recommendation: HTTP content negotiation
  * Like "I prefer JSON, but can accept XML"
* Pagination support
  * max per page
  * start page
  * Default value defined

### Tea API definition

The API will be defined in an OpenAPI format, specifying
methods, objects and responses.
