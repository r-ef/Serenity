#[cfg(test)]
mod tests {
    use reqwest::Client;
    use tokio;
    use std::env;

    #[tokio::test]
    async fn test_mining_endpoint() {
        let client = Client::new();
        let base_url = env::var("SERENITY_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
        let url = format!("{}/mine", base_url);

        let response = client.post(url)
            .body(
                r#"{
                    "address": "blah"
                }"#
            )
            .send()
            .await
            .expect("Failed to send request");

        // assert_eq!(response.status(), 200);

        let body = response.text().await.expect("Failed to read response body");
        println!("Response body: {}", body);

    }
}