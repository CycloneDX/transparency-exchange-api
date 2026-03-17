package dependency_policy

import future.keywords

# Main decision: allow if all conditions met

allow := true if {

    not any_retired

    not any_critical_vuln_without_exception

    not any_quarantined_without_exception

    input.snapshot_digest != ""

    input.attestation.no_network == true

}

# Helper: Check for any retired deps

any_retired if {

    some node in input.deps_graph.nodes

    node.lifecycle_state == "RETIRED"

}

# Helper: Critical vuln without exception

any_critical_vuln_without_exception if {

    some vuln in input.advisories

    vuln.severity == "CRITICAL"

    not exception_exists(vuln.purl, input.exceptions)

}

# Helper: Quarantined without exception

any_quarantined_without_exception if {

    some node in input.deps_graph.nodes

    node.lifecycle_state == "QUARANTINED"

    now_ns() > time.parse_rfc3339_ns(node.sunset_date)

    not exception_exists(node.purl, input.exceptions)

}

# Helper: Check if exception exists

exception_exists(purl, exceptions) if {

    some exc in exceptions

    exc.purl == purl

    now_ns() < time.parse_rfc3339_ns(exc.expiry)

    exc.justified == true

}

# Note: This is a skeleton; extend for deprecated warnings, license gates, max depth, cycle detection, etc.
