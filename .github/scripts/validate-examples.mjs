#!/usr/bin/env node
// Validates every example in the TEA OpenAPI document and in the .well-known
// discovery schema against the schema it illustrates.
//
// OpenAPI: schema `examples` / `example` at any depth (component schemas and the
// schemas of parameters, headers and media types), and media type, parameter and
// header `examples` / `example`. Discovery: the schema's own `examples`.

import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import Ajv from "ajv";
import Ajv2020 from "ajv/dist/2020.js";
import addFormats from "ajv-formats";
import { parse } from "yaml";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const OPENAPI_FILE = "spec/openapi.yaml";
const WELL_KNOWN_FILE = "discovery/tea-well-known.schema.json";
const OPENAPI_ID = "https://tea.invalid/openapi.json";

const AJV_OPTIONS = { strict: false, allErrors: true };

const SCHEMA_CHILD = [
  "items", "additionalProperties", "not", "if", "then", "else", "contains",
  "propertyNames", "unevaluatedItems", "unevaluatedProperties",
];
const SCHEMA_CHILD_LIST = ["allOf", "anyOf", "oneOf", "prefixItems"];
const SCHEMA_CHILD_MAP = ["properties", "patternProperties", "dependentSchemas", "$defs"];

const failures = [];
let checked = 0;

const isObject = (v) => v !== null && typeof v === "object" && !Array.isArray(v);

function pointer(segments) {
  return "#" + segments
    .map((s) => "/" + encodeURIComponent(String(s).replace(/~/g, "~0").replace(/\//g, "~1")))
    .join("");
}

function describe(errors) {
  return errors
    .map((e) => `    ${e.instancePath || "(root)"} ${e.message}${e.params?.allowedValues ? `: ${e.params.allowedValues.join(", ")}` : ""}`)
    .join("\n");
}

function check(file, validate, value, where) {
  checked++;
  if (!validate(value)) {
    failures.push(`${file} ${where}\n${describe(validate.errors)}`);
  }
}

// ---- OpenAPI --------------------------------------------------------------

const doc = parse(readFileSync(path.join(ROOT, OPENAPI_FILE), "utf-8"));
doc.$id = OPENAPI_ID;
const ajv2020 = new Ajv2020(AJV_OPTIONS);
addFormats(ajv2020);
ajv2020.addSchema(doc, OPENAPI_ID, undefined, false);

const validators = new Map();
function validatorAt(segments) {
  const ref = OPENAPI_ID + pointer(segments);
  if (!validators.has(ref)) validators.set(ref, ajv2020.compile({ $ref: ref }));
  return validators.get(ref);
}

function resolveExample(example) {
  const ref = example?.$ref;
  if (typeof ref !== "string" || !ref.startsWith("#/components/examples/")) return example;
  return doc.components.examples[ref.slice("#/components/examples/".length)];
}

function checkSchemaExamples(schema, segments) {
  if (Array.isArray(schema.examples)) {
    schema.examples.forEach((value, i) =>
      check(OPENAPI_FILE, validatorAt(segments), value, `${pointer([...segments, "examples", i])}`));
  }
  if ("example" in schema) {
    check(OPENAPI_FILE, validatorAt(segments), schema.example, pointer([...segments, "example"]));
  }
}

function walkSchema(schema, segments) {
  if (!isObject(schema)) return;
  checkSchemaExamples(schema, segments);
  for (const key of SCHEMA_CHILD) walkSchema(schema[key], [...segments, key]);
  for (const key of SCHEMA_CHILD_LIST) {
    if (Array.isArray(schema[key])) schema[key].forEach((s, i) => walkSchema(s, [...segments, key, i]));
  }
  for (const key of SCHEMA_CHILD_MAP) {
    if (isObject(schema[key])) {
      for (const [name, s] of Object.entries(schema[key])) walkSchema(s, [...segments, key, name]);
    }
  }
}

// Media type, parameter and header objects carry a `schema` next to their own
// `examples` map (of Example Objects) or single `example`.
function checkObjectExamples(node, segments) {
  const schemaSegments = [...segments, "schema"];
  if (isObject(node.examples)) {
    for (const [name, raw] of Object.entries(node.examples)) {
      const example = resolveExample(raw);
      if (isObject(example) && "value" in example) {
        check(OPENAPI_FILE, validatorAt(schemaSegments), example.value,
          pointer([...segments, "examples", name, "value"]));
      }
    }
  }
  if ("example" in node) {
    check(OPENAPI_FILE, validatorAt(schemaSegments), node.example, pointer([...segments, "example"]));
  }
}

function walkDocument(node, segments) {
  if (Array.isArray(node)) {
    node.forEach((child, i) => walkDocument(child, [...segments, i]));
    return;
  }
  if (!isObject(node)) return;
  if (isObject(node.schema)) {
    walkSchema(node.schema, [...segments, "schema"]);
    checkObjectExamples(node, segments);
  }
  for (const [key, child] of Object.entries(node)) {
    if (key === "schema" || key === "examples" || key === "example") continue;
    if (segments.length === 1 && segments[0] === "components" && key === "schemas") continue;
    walkDocument(child, [...segments, key]);
  }
}

for (const [name, schema] of Object.entries(doc.components?.schemas ?? {})) {
  walkSchema(schema, ["components", "schemas", name]);
}
walkDocument(doc, []);

// ---- .well-known discovery schema -----------------------------------------

const wellKnown = JSON.parse(readFileSync(path.join(ROOT, WELL_KNOWN_FILE), "utf-8"));
const ajv07 = new Ajv(AJV_OPTIONS);
addFormats(ajv07);
const validateWellKnown = ajv07.compile(wellKnown);
(wellKnown.examples ?? []).forEach((value, i) =>
  check(WELL_KNOWN_FILE, validateWellKnown, value, `#/examples/${i}`));

// ---- Report ---------------------------------------------------------------

if (failures.length) {
  console.error(`${failures.length} of ${checked} examples do not match their schema:\n`);
  console.error(failures.join("\n\n"));
  process.exit(1);
}
console.log(`All ${checked} examples match their schema.`);
