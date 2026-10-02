use serde_json::{Value, json};

#[tokio::main]
async fn main() {
    let key_path = std::env::args().nth(1).unwrap();
    let key_file = std::fs::File::open(key_path).unwrap();
    let keys = serde_json::from_reader::<_, Keys>(&key_file).unwrap();
    let from_yandex = get_token(keys.jwt()).await.unwrap();
    println!("Reply for IAM-token request: {:?}", from_yandex);
}

pub async fn get_token(jwt: impl AsRef<str>) -> Option<String> {
    const YANDEX_FETCH_IAM_URL: &str = "https://iam.api.cloud.yandex.net/iam/v1/tokens";
    const IAM_TOKEN_KEY: &str = "iamToken";

    let client = reqwest::Client::new();
    let req = client
        .post(YANDEX_FETCH_IAM_URL)
        .json(&serde_json::json!({"jwt":jwt.as_ref()}));
    println!("sending {req:?}");
    let resp = req
        .send()
        .await
        .inspect_err(|e| {
            println!("Error sending request for IAM-token: {:?}", e);
        })
        .ok()?;
    println!("has resp={resp:?}");
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
                "No valid {:?} in response from token server {}",
                IAM_TOKEN_KEY,
                serde_json::to_string_pretty(&json).unwrap()
            );
            None
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[allow(unused)]
struct Keys {
    created_at: String,
    id: String,
    service_account_id: String,
    key_algorithm: String,
    public_key: String,
    private_key: String,
}

impl Keys {
    pub fn jwt(&self) -> String {
        let encoding_key =
            jsonwebtoken::EncodingKey::from_rsa_pem(self.private_key.as_bytes()).unwrap();
        let jwt = jsonwebtoken::encode(
            &jsonwebtoken::Header {
                alg: jsonwebtoken::Algorithm::PS256,
                kid: Some(self.id.to_owned()),
                ..Default::default()
            },
            &self.claims(),
            &encoding_key,
        );
        println!("Key: {jwt:?}",);
        jwt.unwrap()
    }

    fn claims(&self) -> Value {
        let now = jsonwebtoken::get_current_timestamp();
        json!({
            "iss": &self.service_account_id,
            "aud": "https://iam.api.cloud.yandex.net/iam/v1/tokens",
            "iat": now,
            "exp": now+60
        })
    }
}
