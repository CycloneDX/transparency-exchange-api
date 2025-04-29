# The TEA product API

After TEA discovery, a user has a URL to the TEA product API where
the TEI is used to query for data. The TEI marks the product sold,
which can be a single unit or multiple units in a bundle.

- For a single product, the output will be metadata about the
  product and a TEA COMPONENT object.
- For a composed product consisting of a bundle, the response
  will be multiple TEA COMPONENT objects.

In addition, all known TEIs for the product will be returned,
in order for a TEA client to avoid duplication.

## Authorization

Authorization can be done on multiple levels, including
which products and versions are supported for a specific user.

## Composite products

If a product consists of a set of products, each with a different
version number and update scheme, a TEA bundle will be the starting
point of discovery. The TEA bundle will list all included parts
and include pointers (URLs) to the TEA index for these.

The URL can be to a different vendor or different site with the
same vendor.

## TEA Product object

- __uuid__: A unique identifier for this product
- __name__: Product name in clear text (str)
- __identifiers__: A list of TEIs that apply to this product
   - __type__: Type of identifier - one of "tei", "purl", or "cpe"
   - __id__: The complete identifier (str)
- __leaves__: A list of product leaves
   - __uuid__: TEA COMPONENT UUID

The TEA Component UUID is used in the Component API to find out which versions
of the Component that exists.

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

### Tea API operation

* Recommendation: Support HTTP compression
* Recommendation: HTTP content negotiation
  * Like "I prefer JSON, but can accept XML"
* Pagination support
  * max per page
  * start page
  * Default value defined


