#[tokio::main]
async fn main() {
    let oauth = std::env::args().nth(1).unwrap();
    let from_yandex = get_token(oauth).await.unwrap();
    println!("Reply for IAM-token request: {:?}", from_yandex);
}

pub async fn get_token(oauth: impl AsRef<str>) -> Option<String> {
    const YANDEX_FETCH_IAM_URL: &str = "https://iam.api.cloud.yandex.net/iam/v1/tokens";
    const IAM_TOKEN_KEY: &str = "iamToken";
    const YANDEX_PASSPORT: &str = "yandexPassportOauthToken";

    let oauth = oauth.as_ref();
    let client = reqwest::Client::new();
    let query = [(YANDEX_PASSPORT, oauth)];
    let req = client.post(YANDEX_FETCH_IAM_URL).query(&query);
    let resp = req
        .send()
        .await
        .inspect_err(|e| {
            println!("Error sending request for IAM-token: {:?}", e);
        })
        .ok()?;
    let json: serde_json::Value = resp
        .json()
        .await
        .inspect_err(|e| {
            println!("Response with IAM-token failed, JSON-error {:?}", e);
        })
        .ok()?;
    match &json[IAM_TOKEN_KEY] {
        serde_json::Value::String(iam) => Some(iam.to_owned()),
        _ => {
            println!(
                "No valid {:?} in response from token server {:?}",
                IAM_TOKEN_KEY, json
            );
            None
        }
    }
}
