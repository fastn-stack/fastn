//! Minimal end-to-end P2P networking test

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

/// Application-specific protocols - meaningful names instead of Ping/Http lies!
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum AppProtocol {
    Echo,
    Math,
}

impl std::fmt::Display for AppProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// Echo Protocol (simple text echo)
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct EchoResponse {
    pub echo: String,
    pub length: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoError {
    pub code: u32,
    pub message: String,
}

type EchoResult = Result<EchoResponse, EchoError>;

// Math Protocol (arithmetic operations)
#[derive(Serialize, Deserialize, Debug)]
pub struct MathRequest {
    pub operation: String, // "add", "multiply", "divide"
    pub a: f64,
    pub b: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MathResponse {
    pub result: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MathError {
    pub error_type: String, // "invalid_operation", "division_by_zero"
    pub details: String,
}

#[allow(dead_code)]
type MathResult = Result<MathResponse, MathError>;

// ============================================================================
// SERVER IMPLEMENTATION
// ============================================================================

/// Multi-protocol P2P server that handles Echo and Math requests
#[allow(dead_code)]
async fn run_multi_protocol_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting multi-protocol P2P server");

    let secret_key = fastn_id52::SecretKey::generate();
    let public_key = secret_key.public_key();

    println!("📡 Server listening on: {public_key}");

    // Listen on both Echo and Math protocols - meaningful names!
    let protocols = vec![AppProtocol::Echo, AppProtocol::Math];

    let mut stream = fastn_p2p::listen!(secret_key, &protocols);

    let mut request_count = 0;

    while let Some(request) = stream.next().await {
        let peer_request = request?;
        request_count += 1;

        println!(
            "📨 Request #{request_count}: {} from {}",
            peer_request.protocol(),
            peer_request.peer()
        );

        // Route based on protocol - clean meaningful names!
        let result = match peer_request.protocol {
            AppProtocol::Echo => {
                // Handle Echo protocol using clean function reference
                peer_request.handle(echo_handler).await
            }
            AppProtocol::Math => {
                // Handle Math protocol using clean function reference
                peer_request.handle(math_handler).await
            }
        };

        if let Err(e) = result {
            eprintln!("❌ Request failed: {e}");
        }

        // Stop after handling some requests for this demo
        if request_count >= 10 {
            println!("✅ Handled {request_count} requests, shutting down");
            break;
        }
    }

    Ok(())
}

/// Echo request handler - returns the result directly
async fn echo_handler(request: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("🔄 Processing echo: '{}'", request.message);

    // Simple echo logic with validation
    if request.message.is_empty() {
        return Err(EchoError {
            code: 400,
            message: "Empty message not allowed".to_string(),
        });
    }

    if request.message.len() > 1000 {
        return Err(EchoError {
            code: 413,
            message: "Message too long".to_string(),
        });
    }

    // Successful response
    Ok(EchoResponse {
        echo: format!("Echo: {}", request.message),
        length: request.message.len(),
    })
}

/// Math request handler - returns the result directly
#[allow(dead_code)]
async fn math_handler(request: MathRequest) -> Result<MathResponse, MathError> {
    println!(
        "🧮 Processing math: {} {} {}",
        request.a, request.operation, request.b
    );

    // Math operation logic
    let result = match request.operation.as_str() {
        "add" => request.a + request.b,
        "multiply" => request.a * request.b,
        "divide" => {
            if request.b == 0.0 {
                return Err(MathError {
                    error_type: "division_by_zero".to_string(),
                    details: "Cannot divide by zero".to_string(),
                });
            }
            request.a / request.b
        }
        unknown => {
            return Err(MathError {
                error_type: "invalid_operation".to_string(),
                details: format!("Unknown operation: {}", unknown),
            });
        }
    };

    // Successful response
    Ok(MathResponse { result })
}

// ============================================================================
// CLIENT IMPLEMENTATION
// ============================================================================

/// Client that makes requests to both protocols
#[allow(dead_code)]
async fn run_test_client(
    server_public_key: &fastn_id52::PublicKey,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📞 Starting test client");

    let client_secret = fastn_id52::SecretKey::generate();

    // Test Echo protocol
    println!("🔄 Testing Echo protocol...");
    let echo_request = EchoRequest {
        message: "Hello, P2P world!".to_string(),
    };

    let echo_result: EchoResult = fastn_p2p::call(
        client_secret.clone(),
        server_public_key,
        AppProtocol::Echo,
        echo_request,
    )
    .await?;

    match echo_result {
        Ok(response) => {
            println!(
                "✅ Echo response: '{}' (length: {})",
                response.echo, response.length
            );
            assert_eq!(response.echo, "Echo: Hello, P2P world!");
        }
        Err(error) => {
            eprintln!("❌ Echo error {}: {}", error.code, error.message);
        }
    }

    // Test Math protocol
    println!("🧮 Testing Math protocol...");
    let math_request = MathRequest {
        operation: "multiply".to_string(),
        a: 6.0,
        b: 7.0,
    };

    let math_result: MathResult = fastn_p2p::call(
        client_secret.clone(),
        server_public_key,
        AppProtocol::Math,
        math_request,
    )
    .await?;

    match math_result {
        Ok(response) => {
            println!("✅ Math response: {} * {} = {}", 6.0, 7.0, response.result);
            assert_eq!(response.result, 42.0);
        }
        Err(error) => {
            eprintln!("❌ Math error {}: {}", error.error_type, error.details);
        }
    }

    // Test error case - division by zero
    println!("🧮 Testing Math error case...");
    let error_request = MathRequest {
        operation: "divide".to_string(),
        a: 10.0,
        b: 0.0,
    };

    let error_result: MathResult = fastn_p2p::call(
        client_secret,
        server_public_key,
        AppProtocol::Math,
        error_request,
    )
    .await?;

    match error_result {
        Ok(response) => {
            eprintln!("❌ Expected error but got response: {}", response.result);
        }
        Err(error) => {
            println!(
                "✅ Expected error received: {} - {}",
                error.error_type, error.details
            );
            assert_eq!(error.error_type, "division_by_zero");
        }
    }

    Ok(())
}

// ============================================================================
// EXAMPLE MAIN FUNCTION (not a test, just shows the pattern)
// ============================================================================

/// Example of how a real application might structure P2P communication
/// This shows the clean API patterns we've achieved
#[allow(dead_code)]
async fn example_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate server key that client will connect to
    let server_secret = fastn_id52::SecretKey::generate();
    let server_public = server_secret.public_key();

    println!("🔑 Server key: {}", server_public.id52());

    // Server setup (in real app, server would use its own secret key)
    let server_task = tokio::spawn(async { run_multi_protocol_server().await });

    // Give server time to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Client interactions - pass server's public key
    let client_task = tokio::spawn(async move { run_test_client(&server_public).await });

    // Wait for completion
    let (server_result, client_result) = tokio::join!(server_task, client_task);

    server_result??;
    client_result??;

    println!("🎉 Multi-protocol P2P test completed successfully!");
    Ok(())
}

// ============================================================================
// LOAD TEST FUNCTION
// ============================================================================

/// High-load test: alternates between protocols for 100 requests
#[allow(dead_code)]
async fn load_test_100_requests(
    server_public_key: &fastn_id52::PublicKey,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting 100-request load test");

    let client_secret = fastn_id52::SecretKey::generate();

    for i in 1..=100 {
        if i % 2 == 0 {
            // Even requests: Echo protocol
            let request = EchoRequest {
                message: format!("Request #{}", i),
            };

            let result: EchoResult = fastn_p2p::call(
                client_secret.clone(),
                server_public_key,
                AppProtocol::Echo,
                request,
            )
            .await?;

            match result {
                Ok(response) => println!("✅ Echo #{}: {}", i, response.echo),
                Err(error) => eprintln!("❌ Echo #{} error: {}", i, error.message),
            }
        } else {
            // Odd requests: Math protocol
            let request = MathRequest {
                operation: "add".to_string(),
                a: i as f64,
                b: 100.0,
            };

            let result: MathResult = fastn_p2p::call(
                client_secret.clone(),
                server_public_key,
                AppProtocol::Math,
                request,
            )
            .await?;

            match result {
                Ok(response) => println!("✅ Math #{}: {} + 100 = {}", i, i, response.result),
                Err(error) => eprintln!("❌ Math #{} error: {}", i, error.details),
            }
        }
    }

    println!("🎉 Completed 100 requests successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_end_to_end_p2p_networking() {
        // Test actual P2P networking: one server, one client, one message
        println!("🚀 Starting end-to-end P2P networking test");

        let server_secret = fastn_id52::SecretKey::generate();
        let server_public = server_secret.public_key();
        let client_secret = fastn_id52::SecretKey::generate();

        println!("📡 Server: {}", server_public.id52());
        println!("📞 Client: {}", client_secret.public_key().id52());

        // Start server task
        let server_task = {
            let server_secret_clone = server_secret.clone();
            tokio::spawn(async move {
                let protocols = vec![AppProtocol::Echo];
                let mut stream = fastn_p2p::listen!(server_secret_clone, &protocols);

                println!("🎧 Server listening for Echo protocol...");

                // Handle exactly one request
                if let Some(request) = stream.next().await {
                    let request = request?;
                    println!(
                        "📨 Server received {} request from {}",
                        request.protocol,
                        request.peer().id52()
                    );

                    // Handle the echo request
                    request.handle(echo_handler).await?;
                    println!("✅ Server handled request successfully");
                }

                Result::<(), Box<dyn std::error::Error + Send + Sync>>::Ok(())
            })
        };

        // Give server time to start listening
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        // Send one message from client to server
        let client_task = {
            let server_public_clone = server_public;
            tokio::spawn(async move {
                println!("📤 Client sending echo request...");

                let request = EchoRequest {
                    message: "Hello P2P world!".to_string(),
                };

                let result: EchoResult = fastn_p2p::call(
                    client_secret,
                    &server_public_clone,
                    AppProtocol::Echo,
                    request,
                )
                .await?;

                match result {
                    Ok(response) => {
                        println!("✅ Client received response: '{}'", response.echo);
                        assert_eq!(response.echo, "Echo: Hello P2P world!");
                        assert_eq!(response.length, 16);
                    }
                    Err(error) => {
                        panic!(
                            "❌ Client received error: {} (code: {})",
                            error.message, error.code
                        );
                    }
                }

                Result::<(), Box<dyn std::error::Error + Send + Sync>>::Ok(())
            })
        };

        // Wait for both tasks to complete
        let (server_result, client_result) = tokio::join!(server_task, client_task);

        server_result.unwrap().unwrap();
        client_result.unwrap().unwrap();

        println!("🎉 End-to-end P2P networking test completed successfully!");
    }
}
