//! Test harness for TEA API implementations.
//!
//! This module provides a test harness that can be used to verify that a TEA
//! server implementation conforms to the TEA specification. The harness includes
//! tests for basic API functionality, error handling, and conformance to
//! defined standards.

use std::collections::HashMap;
use reqwest::Client;

/// Result of a harness test.
#[derive(Debug)]
pub struct HarnessResult {
    /// Test name.
    pub test_name: String,
    /// Whether the test passed.
    pub passed: bool,
    /// Optional error message.
    pub error: Option<String>,
}

/// TEA API Test Harness.
pub struct TeaHarness {
    /// Base URL of the TEA server to test.
    base_url: String,
    /// HTTP client.
    client: Client,
}

impl TeaHarness {
    /// Create a new harness for the given base URL.
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    /// Run all harness tests.
    pub async fn run_all_tests(&self) -> Vec<HarnessResult> {
        let mut results = Vec::new();

        results.push(self.test_health_endpoint().await);
        results.push(self.test_discovery_endpoint().await);
        results.push(self.test_product_listing().await);
        // Add more tests as needed

        results
    }

    /// Test the health endpoint.
    async fn test_health_endpoint(&self) -> HarnessResult {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => HarnessResult {
                test_name: "Health Endpoint".to_string(),
                passed: true,
                error: None,
            },
            Ok(response) => HarnessResult {
                test_name: "Health Endpoint".to_string(),
                passed: false,
                error: Some(format!("Unexpected status: {}", response.status())),
            },
            Err(e) => HarnessResult {
                test_name: "Health Endpoint".to_string(),
                passed: false,
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }

    /// Test the discovery endpoint.
    async fn test_discovery_endpoint(&self) -> HarnessResult {
        let url = format!("{}/discovery", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => HarnessResult {
                test_name: "Discovery Endpoint".to_string(),
                passed: true,
                error: None,
            },
            Ok(response) => HarnessResult {
                test_name: "Discovery Endpoint".to_string(),
                passed: false,
                error: Some(format!("Unexpected status: {}", response.status())),
            },
            Err(e) => HarnessResult {
                test_name: "Discovery Endpoint".to_string(),
                passed: false,
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }

    /// Test product listing endpoint.
    async fn test_product_listing(&self) -> HarnessResult {
        let url = format!("{}/products", self.base_url);
        match self.client.get(&url).send().await {
            Ok(response) if response.status().is_success() => HarnessResult {
                test_name: "Product Listing".to_string(),
                passed: true,
                error: None,
            },
            Ok(response) => HarnessResult {
                test_name: "Product Listing".to_string(),
                passed: false,
                error: Some(format!("Unexpected status: {}", response.status())),
            },
            Err(e) => HarnessResult {
                test_name: "Product Listing".to_string(),
                passed: false,
                error: Some(format!("Request failed: {}", e)),
            },
        }
    }

    /// Print test results.
    pub fn print_results(&self, results: &[HarnessResult]) {
        println!("TEA API Test Harness Results:");
        println!("==============================");

        let passed = results.iter().filter(|r| r.passed).count();
        let total = results.len();

        for result in results {
            let status = if result.passed { "PASS" } else { "FAIL" };
            println!("{}: {}", result.test_name, status);
            if let Some(error) = &result.error {
                println!("  Error: {}", error);
            }
        }

        println!("\nSummary: {}/{} tests passed", passed, total);
        if passed == total {
            println!("All tests passed! 🎉");
        } else {
            println!("Some tests failed. Check the implementation.");
        }
    }
}

/// Convenience function to run the harness on a TEA server.
pub async fn run_harness(base_url: &str) {
    let harness = TeaHarness::new(base_url.to_string());
    let results = harness.run_all_tests().await;
    harness.print_results(&results);
}
