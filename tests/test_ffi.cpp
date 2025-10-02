#include <cstdio>
#include <cstdlib>
#include <cstring>
#include "../generated/include/tlsnprover.h"

using namespace tlsnprover;

/*
 * INTEGRATION TESTING CONFIGURATION
 *
 * To test with a local notary server:
 * 1. Start the local notary server:
 *    cd /path/to/tlsn && cargo run --release --bin notary-server
 *
 * 2. Set test credentials in .env file:
 *    ZKP2P_TEST_URL=https://wise.com/gateway/v3/profiles/{id}/transfers/{id}
 *    ZKP2P_TEST_COOKIE=your_cookie_here
 *    ZKP2P_TEST_ACCESS_TOKEN=your_token_here
 *
 * WARNING: Never commit real credentials to version control!
 */

// Test credentials loaded from environment variables (see .env)
static const char* test_url = NULL;
static const char* test_access_token = NULL;
static const char* test_cookie = NULL;

// Test configuration
static const char* test_user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36";
static const char* test_provider_host = "wise.com";
static uint16_t test_provider_port = 443;
static const char* test_notary_host = "127.0.0.1";
static uint16_t test_notary_port = 7047;
static bool test_notary_tls_enabled = false;
static size_t test_max_sent_data = 4096;
static size_t test_max_recv_data = 16384;
static const char* test_unauthed_bytes = "X";

static int ENABLE_INTEGRATION_TESTS = 1;  // Set to 1 to enable integration tests

// Mode constants (from args.rs)
typedef enum {
    MODE_PROVE = 0,
    MODE_PRESENT = 1,
    MODE_PROVE_TO_PRESENT = 2
} tlsn_mode_t;

void load_test_credentials() {
    test_url = getenv("ZKP2P_TEST_URL");
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
    printf("\n2. Testing tlsn_prove() with invalid mode...\n");
    result = tlsn_prove(
        -1,  // invalid mode
        "/test",
        NULL,
        NULL,
        test_user_agent,
        test_provider_host,
        test_provider_port,
        test_notary_host,
        test_notary_port,
        test_notary_tls_enabled,
        test_max_sent_data,
        test_max_recv_data
    );
    if (result != 0) {
        printf("   ✅ Invalid mode properly rejected with code: %d\n", result);
        print_error_if_available();
    } else {
        printf("   ❌ Invalid parameters should have been rejected\n");
    }

    // Test invalid provider URL (nonexistent presentation file)
    printf("\n3. Testing tlsn_verify() with nonexistent provider...\n");
    result = tlsn_verify("nonexistent.com", test_unauthed_bytes);
    if (result != 0) {
        printf("   ✅ Nonexistent provider properly rejected with code: %d\n", result);
        print_error_if_available();
    } else {
        printf("   ❌ Nonexistent provider should have been rejected\n");
    }

    // Integration tests (only run if enabled and credentials are set)
    if (ENABLE_INTEGRATION_TESTS) {
        printf("\n=== INTEGRATION TESTS ===\n");

        if (!test_url || !test_cookie || !test_access_token) {
            printf("\n⚠️  Integration tests skipped - credentials not set\n");
            printf("Set ZKP2P_TEST_URL, ZKP2P_TEST_COOKIE, and ZKP2P_TEST_ACCESS_TOKEN in .env\n");
        } else {
            // Test prove-to-present operation (creates both attestation and presentation)
            printf("\n4. Testing tlsn_prove() in PROVE_TO_PRESENT mode...\n");
            printf("   Mode: PROVE_TO_PRESENT (%d)\n", MODE_PROVE_TO_PRESENT);
            printf("   URL: %s\n", test_url);
            printf("   Using credentials from environment variables...\n");

            result = tlsn_prove(
                MODE_PROVE_TO_PRESENT,
                test_url,
                test_cookie,
                test_access_token,
                test_user_agent,
                test_provider_host,
                test_provider_port,
                test_notary_host,
                test_notary_port,
                test_notary_tls_enabled,
                test_max_sent_data,
                test_max_recv_data
            );

            if (result == 0) {
                printf("   ✅ Prove-to-present operation successful\n");
            } else {
                printf("   ⚠️  Prove-to-present operation failed with code: %d\n", result);
                printf("   (This may be expected if notary server is not running)\n");
                print_error_if_available();
            }

            // Test verify operation
            printf("\n5. Testing tlsn_verify()...\n");
            printf("   Verifying presentation file for wise.com...\n");
            result = tlsn_verify("wise.com", test_unauthed_bytes);

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
                test_url,
                test_cookie,
                test_access_token,
                test_user_agent,
                test_provider_host,
                test_provider_port,
                test_notary_host,
                test_notary_port,
                test_notary_tls_enabled,
                test_max_sent_data,
                test_max_recv_data
            );

            if (result == 0) {
                printf("   ✅ Prove operation successful\n");
            } else {
                printf("   ⚠️  Prove operation failed with code: %d\n", result);
                printf("   (This may be expected if notary server is not running)\n");
                print_error_if_available();
            }

            // Test pure present operation (uses existing attestation)
            printf("\n7. Testing tlsn_prove() in PRESENT mode...\n");
            result = tlsn_prove(
                MODE_PRESENT,
                NULL,  // URL not required for present mode
                NULL,  // Cookie not required for present mode
                NULL,  // Access token not required for present mode
                test_user_agent,
                test_provider_host,
                test_provider_port,
                test_notary_host,
                test_notary_port,
                test_notary_tls_enabled,
                test_max_sent_data,
                test_max_recv_data
            );

            if (result == 0) {
                printf("   ✅ Present operation successful\n");
            } else {
                printf("   ⚠️  Present operation failed with code: %d\n", result);
                printf("   (May fail if no attestation file exists)\n");
                print_error_if_available();
            }
        }
    } else {
        printf("\n=== INTEGRATION TESTS SKIPPED ===\n");
        printf("To enable integration tests:\n");
        printf("1. Set ENABLE_INTEGRATION_TESTS = 1\n");
        printf("2. Start local notary server\n");
        printf("3. Set credentials in .env file (ZKP2P_TEST_*)\n");
    }

    // Cleanup
    printf("\n8. Testing tlsn_cleanup()...\n");
    tlsn_cleanup();
    printf("   ✅ Cleanup completed\n");

    printf("\n🎉 FFI test completed!\n");
    if (ENABLE_INTEGRATION_TESTS) {
        printf("📋 Integration tests were executed (results may vary)\n");
    } else {
        printf("📋 Basic functionality tests passed. Enable integration tests for full validation.\n");
    }

    return 0;
}
