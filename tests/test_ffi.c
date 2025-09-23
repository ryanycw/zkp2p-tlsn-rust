#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../include/tlsnprover.h"

/*
 * INTEGRATION TESTING CONFIGURATION
 *
 * To test with a local notary server:
 * 1. Start the local notary server:
 *    cd /path/to/tlsn && cargo run --release --bin notary-server
 *
 * 2. Configure .env for local testing:
 *    NOTARY_HOST=127.0.0.1
 *    NOTARY_PORT=7047
 *    NOTARY_TLS=false
 *
 * 3. Replace MOCK variables below with real Wise.com credentials:
 *    - Get your profile_id from Wise account settings
 *    - Get transaction_id from a completed transaction
 *    - Extract cookie from browser dev tools (Application -> Cookies)
 *    - Extract access_token from API calls in Network tab
 *
 * WARNING: Never commit real credentials to version control!
 */

// Test credentials loaded from environment variables (see .env)
static const char* test_profile_id = NULL;
static const char* test_transaction_id = NULL;
static const char* test_access_token = NULL;
static const char* test_cookie = NULL;

// Test configuration
static int ENABLE_INTEGRATION_TESTS = 1;  // Set to 1 to enable integration tests

// Provider constants (from args.rs)
typedef enum {
    PROVIDER_WISE = 0,
    PROVIDER_PAYPAL = 1
} tlsn_provider_t;

// Mode constants (from args.rs)
typedef enum {
    MODE_PROVE = 0,
    MODE_PRESENT = 1,
    MODE_PROVE_TO_PRESENT = 2
} tlsn_mode_t;

void load_test_credentials() {
    test_profile_id = getenv("ZKP2P_TEST_PROFILE_ID");
    test_transaction_id = getenv("ZKP2P_TEST_TRANSACTION_ID");
    test_access_token = getenv("ZKP2P_TEST_ACCESS_TOKEN");
    test_cookie = getenv("ZKP2P_TEST_COOKIE");
}

void print_error_if_available() {
    const char* error = tlsn_get_last_error();
    if (error) {
        printf("   Error: %s\n", error);
        tlsn_free_error_string((char*)error);
    }
}

int main() {
    printf("Testing ZKP2P TLSNotary FFI...\n");
    printf("Integration tests: %s\n\n", ENABLE_INTEGRATION_TESTS ? "ENABLED" : "DISABLED");

    // Load test credentials from environment variables
    load_test_credentials();

    // Test initialization
    printf("1. Testing tlsn_init()...\n");
    int32_t result = tlsn_init();
    if (result == 0) {
        printf("   ✅ Initialization successful\n");
    } else {
        printf("   ❌ Initialization failed with code: %d\n", result);
        print_error_if_available();
        return 1;
    }

    // Test invalid parameters
    printf("\n2. Testing tlsn_prove() with invalid parameters...\n");
    result = tlsn_prove(-1, 0, "test", NULL, NULL, NULL);
    if (result != 0) {
        printf("   ✅ Invalid mode properly rejected with code: %d\n", result);
        print_error_if_available();
    } else {
        printf("   ❌ Invalid parameters should have been rejected\n");
    }

    // Test invalid provider
    printf("\n3. Testing tlsn_verify() with invalid provider...\n");
    result = tlsn_verify(99, test_transaction_id);
    if (result != 0) {
        printf("   ✅ Invalid provider properly rejected with code: %d\n", result);
        print_error_if_available();
    } else {
        printf("   ❌ Invalid provider should have been rejected\n");
    }

    // Integration tests (only run if enabled and local notary is available)
    if (ENABLE_INTEGRATION_TESTS) {
        printf("\n=== INTEGRATION TESTS ===\n");

        // Test prove-to-present operation (creates both attestation and presentation)
        printf("\n4. Testing tlsn_prove() in PROVE_TO_PRESENT mode...\n");
        printf("   Mode: PROVE_TO_PRESENT (%d), Provider: WISE (%d)\n", MODE_PROVE_TO_PRESENT, PROVIDER_WISE);
        printf("   Profile ID: %s\n", test_profile_id ? test_profile_id : "(not set)");
        printf("   Transaction ID: %s\n", test_transaction_id ? test_transaction_id : "(not set)");
        printf("   Using credentials from environment variables...\n");

        result = tlsn_prove(
            MODE_PROVE_TO_PRESENT,
            PROVIDER_WISE,
            test_transaction_id,
            test_profile_id,
            test_cookie,
            test_access_token
        );

        if (result == 0) {
            printf("   ✅ Prove-to-present operation successful\n");
        } else {
            printf("   ⚠️  Prove-to-present operation failed with code: %d\n", result);
            printf("   (Expected with test credentials from .env)\n");
            print_error_if_available();
        }

        // Test verify operation (should work if prove-to-present succeeded)
        printf("\n5. Testing tlsn_verify() with valid provider...\n");
        result = tlsn_verify(PROVIDER_WISE, test_transaction_id);

        if (result == 0) {
            printf("   ✅ Verify operation successful\n");
        } else {
            printf("   ⚠️  Verify operation failed with code: %d\n", result);
            printf("   (May fail if no presentation file exists)\n");
            print_error_if_available();
        }

        // Test pure prove operation
        printf("\n6. Testing tlsn_prove() in PROVE mode...\n");
        result = tlsn_prove(
            MODE_PROVE,
            PROVIDER_WISE,
            test_transaction_id,
            test_profile_id,
            test_cookie,
            test_access_token
        );

        if (result == 0) {
            printf("   ✅ Prove operation successful\n");
        } else {
            printf("   ⚠️  Prove operation failed with code: %d\n", result);
            printf("   (Expected with test credentials from .env)\n");
            print_error_if_available();
        }

        // Test pure present operation (uses existing attestation)
        printf("\n7. Testing tlsn_prove() in PRESENT mode...\n");
        result = tlsn_prove(
            MODE_PRESENT,
            PROVIDER_WISE,
            test_transaction_id,
            NULL,  // Not required for present mode
            NULL,  // Not required for present mode
            NULL   // Not required for present mode
        );

        if (result == 0) {
            printf("   ✅ Present operation successful\n");
        } else {
            printf("   ⚠️  Present operation failed with code: %d\n", result);
            printf("   (May fail if no attestation file exists)\n");
            print_error_if_available();
        }
    } else {
        printf("\n=== INTEGRATION TESTS SKIPPED ===\n");
        printf("To enable integration tests:\n");
        printf("1. Set ENABLE_INTEGRATION_TESTS = 1\n");
        printf("2. Start local notary server\n");
        printf("3. Set real credentials in .env file (ZKP2P_TEST_*)\n");
        printf("4. Configure .env for local testing\n");
    }

    // Cleanup
    printf("\n8. Testing tlsn_cleanup()...\n");
    tlsn_cleanup();
    printf("   ✅ Cleanup completed\n");

    printf("\n🎉 FFI test completed!\n");
    if (ENABLE_INTEGRATION_TESTS) {
        printf("📋 Integration tests were executed (results may vary with mock data)\n");
    } else {
        printf("📋 Basic functionality tests passed. Enable integration tests for full validation.\n");
    }

    return 0;
}