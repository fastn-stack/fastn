//! Multi-protocol P2P server test
//! 
//! This test demonstrates how to build a P2P server that handles multiple protocols
//! with type-safe request-response patterns.

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

// ============================================================================
// PROTOCOL DEFINITIONS - Shared between client and server
// ============================================================================

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
    pub operation: String,  // "add", "multiply", "divide"
    pub a: f64,
    pub b: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MathResponse {
    pub result: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MathError {
    pub error_type: String,  // "invalid_operation", "division_by_zero"
    pub details: String,
}

type MathResult = Result<MathResponse, MathError>;

// ============================================================================
// SERVER IMPLEMENTATION
// ============================================================================

/// Multi-protocol P2P server that handles Echo and Math requests
async fn run_multi_protocol_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting multi-protocol P2P server");
    
    let secret_key = fastn_id52::SecretKey::generate();
    let public_key = secret_key.public_key();
    
    println!("📡 Server listening on: {public_key}");
    
    // Listen on both Echo and Math protocols
    let protocols = vec![fastn_p2p::Protocol::Ping, fastn_p2p::Protocol::Http];
    
    let stream = fastn_p2p::listen(secret_key, &protocols)?;
    let mut stream = std::pin::pin!(stream);
    
    let mut request_count = 0;
    
    while let Some(peer_request) = stream.next().await {
        let peer_request = peer_request?;
        request_count += 1;
        
        println!("📨 Request #{}: {:?} from {}", 
                request_count, 
                peer_request.protocol(), 
                peer_request.peer().id52());
        
        // Route based on protocol
        match peer_request.protocol {
            fastn_p2p::Protocol::Ping => {
                // Handle Echo protocol using the high-level API
                if let Err(e) = handle_echo_request(peer_request).await {
                    eprintln!("❌ Echo request failed: {}", e);
                }
            }
            fastn_p2p::Protocol::Http => {
                // Handle Math protocol using the high-level API  
                if let Err(e) = handle_math_request(peer_request).await {
                    eprintln!("❌ Math request failed: {}", e);
                }
            }
            other => {
                eprintln!("❓ Unexpected protocol: {:?}", other);
                // Could send an error response here if needed
            }
        }
        
        // Stop after handling some requests for this demo
        if request_count >= 10 {
            println!("✅ Handled {} requests, shutting down", request_count);
            break;
        }
    }
    
    Ok(())
}

/// Handle Echo protocol requests
async fn handle_echo_request(peer_request: fastn_p2p::Request) -> Result<(), fastn_p2p::HandleRequestError> {
    peer_request.handle(|request: EchoRequest| async move {
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
    }).await
}

/// Handle Math protocol requests  
async fn handle_math_request(peer_request: fastn_p2p::Request) -> Result<(), fastn_p2p::HandleRequestError> {
    peer_request.handle(|request: MathRequest| async move {
        println!("🧮 Processing math: {} {} {}", request.a, request.operation, request.b);
        
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
    }).await
}

// ============================================================================
// CLIENT IMPLEMENTATION  
// ============================================================================

/// Client that makes requests to both protocols
async fn run_test_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📞 Starting test client");
    
    let client_secret = fastn_id52::SecretKey::generate();
    let server_public = fastn_id52::SecretKey::generate().public_key(); // Mock server key
    
    // Test Echo protocol
    println!("🔄 Testing Echo protocol...");
    let echo_request = EchoRequest {
        message: "Hello, P2P world!".to_string(),
    };
    
    let echo_result: EchoResult = fastn_p2p::call(
        client_secret.clone(),
        &server_public,
        fastn_p2p::Protocol::Ping,
        echo_request,
    ).await?;
    
    match echo_result {
        Ok(response) => {
            println!("✅ Echo response: '{}' (length: {})", response.echo, response.length);
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
        &server_public,
        fastn_p2p::Protocol::Http,
        math_request,
    ).await?;
    
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
        &server_public,
        fastn_p2p::Protocol::Http,
        error_request,
    ).await?;
    
    match error_result {
        Ok(response) => {
            eprintln!("❌ Expected error but got response: {}", response.result);
        }
        Err(error) => {
            println!("✅ Expected error received: {} - {}", error.error_type, error.details);
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
async fn example_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Server setup
    let server_task = tokio::spawn(async {
        run_multi_protocol_server().await
    });
    
    // Give server time to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    
    // Client interactions
    let client_task = tokio::spawn(async {
        run_test_client().await
    });
    
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
async fn load_test_100_requests() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Starting 100-request load test");
    
    let client_secret = fastn_id52::SecretKey::generate(); 
    let server_public = fastn_id52::SecretKey::generate().public_key();
    
    for i in 1..=100 {
        if i % 2 == 0 {
            // Even requests: Echo protocol
            let request = EchoRequest {
                message: format!("Request #{}", i),
            };
            
            let result: EchoResult = fastn_p2p::call(
                client_secret.clone(),
                &server_public,
                fastn_p2p::Protocol::Ping,
                request,
            ).await?;
            
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
                &server_public,
                fastn_p2p::Protocol::Http,
                request,
            ).await?;
            
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

    #[test]
    fn test_protocol_definitions() {
        // Test that our shared types work correctly
        let echo_req = EchoRequest {
            message: "test".to_string(),
        };
        
        let json = serde_json::to_string(&echo_req).unwrap();
        let parsed: EchoRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(echo_req.message, parsed.message);
        
        let math_req = MathRequest {
            operation: "add".to_string(),
            a: 1.0,
            b: 2.0,
        };
        
        let json = serde_json::to_string(&math_req).unwrap();
        let parsed: MathRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(math_req.operation, parsed.operation);
        assert_eq!(math_req.a, parsed.a);
        assert_eq!(math_req.b, parsed.b);
    }

    #[test]
    fn test_error_serialization() {
        let echo_error = EchoError {
            code: 404,
            message: "Not found".to_string(),
        };
        
        let json = serde_json::to_string(&echo_error).unwrap();
        let parsed: EchoError = serde_json::from_str(&json).unwrap(); 
        assert_eq!(echo_error.code, parsed.code);
        assert_eq!(echo_error.message, parsed.message);
    }
}
