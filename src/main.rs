#[tokio::main]
async fn main() {
    let key_path = std::env::args().nth(1).unwrap();
    let key_file = std::fs::File::open(key_path).unwrap();
    let keys = serde_json::from_reader::<_, Keys>(&key_file).unwrap();
    let from_yandex = get_token(keys.jwt()).await.unwrap();
    println!("Reply for IAM-token request: {:?}", from_yandex);
}

const YANDEX_FETCH_IAM_URL: &str = "https://iam.api.cloud.yandex.net/iam/v1/tokens";

pub async fn get_token(jwt: impl AsRef<str>) -> Option<String> {
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
struct Keys {
    id: String,
    service_account_id: String,
    private_key: String,
}

impl Keys {
    pub fn jwt(&self) -> String {
        let encoding_key =
            jsonwebtoken::EncodingKey::from_rsa_pem(self.private_key.as_bytes()).unwrap();
        let jwt = jsonwebtoken::encode(
            &self.header(),
            &Claims::new(&self.service_account_id),
            &encoding_key,
        );
        println!("Token to exchange: {jwt:?}",);
        jwt.unwrap()
    }

    fn header(&self) -> jsonwebtoken::Header {
        jsonwebtoken::Header {
            alg: jsonwebtoken::Algorithm::PS256,
            kid: Some(self.id.to_owned()),
            ..Default::default()
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct Claims<'a> {
    iss: &'a str,
    aud: &'a str,
    iat: u64,
    exp: u64,
}

impl<'a> Claims<'a> {
    pub fn new(service_account_id: &'a str) -> Self {
        let now = jsonwebtoken::get_current_timestamp();
        Self {
            iss: service_account_id,
            aud: YANDEX_FETCH_IAM_URL,
            iat: now,
            exp: now + 60,
        }
    }
}
