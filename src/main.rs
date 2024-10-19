#[tokio::main]
async fn main() {
    let mut args_list = std::env::args();
    let (_, oauth) = (args_list.next().unwrap(), args_list.next().unwrap());
    println!(
        "Reply for IAM-token request: {:?}",
        get_token(oauth.as_str()).await
    );
}

pub async fn get_token(oauth: &str) -> Option<String> {
    const IAM_TOKEN_KEY: &str = "iamToken";
    const YANDEX_PASSPORT: &str = "yandexPassportOauthToken";

    let client = reqwest::Client::new();
    let query = [(YANDEX_PASSPORT, oauth)];
    let req = client
        .post("https://iam.api.cloud.yandex.net/iam/v1/tokens")
        .query(&query);
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
