#[cfg(test)]
mod tests {
    use reqwest::Client;
    use tokio;

    #[tokio::test]
    async fn test_mining_endpoint() {
        let client = Client::new();

        let url = "http://127.0.0.1:8000/mine";

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