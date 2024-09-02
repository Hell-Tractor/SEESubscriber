use base64::Engine;
use log::debug;
use reqwest::Client;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("Failed to decode RSA public key: {0}")]
    RsaPubKeyDecodeError(#[from] rsa::pkcs8::spki::Error),
    #[error("Failed to encrypt password: {0}")]
    RsaEncryptError(#[from] rsa::errors::Error),
    #[error("Failed to decode base64 to utf8: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("No url params found after redirect")]
    NoUrlParamsError,
    #[error("Url params not found: {0}")]
    UrlParamsNotFoundError(String),
    #[error("Failed to read input: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to send SMS code: {0}")]
    #[allow(dead_code)]
    SMSCodeSendError(String),
    #[error("Field not found in response json: {0}")]
    FieldNotFound(String)
}

type Result<T> = std::result::Result<T, Error>;

static PRE_LOGIN_URL1: &str = "https://1.tongji.edu.cn/api/ssoservice/system/loginIn";
static LOGIN_URL: &str = "https://iam.tongji.edu.cn/idp/authcenter/ActionAuthChain";
static LOGIN_URL2: &str = "https://iam.tongji.edu.cn/idp/AuthnEngine";
static LOGIN_URL3: &str = "https://1.tongji.edu.cn/api/sessionservice/session/login";
#[allow(dead_code)]
static SMS_URL: &str = "https://iam.tongji.edu.cn/idp/sendCheckCode.do";
// static LOGIN_URL: &str = "https://iam.tongji.edu.cn/idp/AuthnEngine";
static RSA_PUB_KEY: &str = "-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQC9t16RqQWUE/J1IyOfoNHc4r/h
6RPnXcWTJ4IbhQVUsEqMMm65F0hiytAgozXmVw68yPJywbpblDrx9zl1wdRcdHCo
UvmPdr9/oCQtpQyVc7BXZIN6wJlD6MTeMeni+N0toNPxfXjiAawjNHGZZuT8wQpN
EMwsVyJ/lonXaVdGZwIDAQAB
-----END PUBLIC KEY-----";
static SP_AUTH_CHAIN_CODE: &str = "4c1eb8ec14fa4e8ba0f31188dbf88cdd";
static CURRENT_AUTH: &str = "urn_oasis_names_tc_SAML_2.0_ac_classes_BAMUsernamePassword";

pub async fn encrypt_password(password: &str) -> Result<String> {
    let mut rng = rand::rngs::OsRng;
    let rsa_pub_key = RsaPublicKey::from_public_key_pem(&RSA_PUB_KEY)?;
    let encrypted_password = rsa_pub_key.encrypt(&mut rng, rsa::Pkcs1v15Encrypt, &password.as_bytes())?;
    let result = base64::engine::general_purpose::STANDARD.encode(&encrypted_password);
    Ok(result)
}

pub async fn login(client: &Client, username: &str, password: &str) -> Result<String> {
    debug!("trying to login with username: {}, password: {}", username, password);
    // make request to get redirect url with param entity_id and authn_lc_key
    let response = client.get(PRE_LOGIN_URL1).send().await?;
    debug!("login 1st request: {:?}", response);
    let params = response.url().query().ok_or(Error::NoUrlParamsError)?;
    let mut entity_id = None;
    let mut authn_lc_key = None;
    params.split("&")
        .filter_map(|param| {
            let mut iter = param.split("=");
            let key = iter.next()?;
            let value = iter.next()?;
            Some((key, value))
        }).for_each(|(key, value)| {
            match key {
                "entityId" => entity_id = Some(value),
                "authnLcKey" => authn_lc_key = Some(value),
                _ => (),
            }
        });
    let entity_id = entity_id.ok_or(Error::UrlParamsNotFoundError("entityId".to_string()))?;
    let authn_lc_key = authn_lc_key.ok_or(Error::UrlParamsNotFoundError("authnLcKey".to_string()))?;
    let encrypted_password = encrypt_password(password).await?;
    debug!("encrypted password: {:?}", encrypted_password);
    let response = client.post(LOGIN_URL)
        .header("Referer", format!("https://iam.tongji.edu.cn/idp/authcenter/ActionAuthChain?entityId={}&authnLcKey={}", entity_id, authn_lc_key))
        .query(&[
            ("authnLcKey", authn_lc_key),
        ]).form(&[
            ("j_username", username),
            ("j_password", &encrypted_password),
            ("j_checkcode", "请输入验证码"),
            ("op", "login"),
            ("spAuthChainCode", SP_AUTH_CHAIN_CODE),
            ("authnLcKey", authn_lc_key),
        ]).send().await?;
    debug!("login 2nd request: {:?}", response);
    // make request to login and get redirect url
    let response = client.post(LOGIN_URL2)
        .header("Referer", format!("https://iam.tongji.edu.cn/idp/authcenter/ActionAuthChain?entityId={}&authnLcKey={}", entity_id, authn_lc_key))
        .query(&[
            ("entityId", entity_id),
            ("currentAuth", CURRENT_AUTH),
            ("authnLcKey", authn_lc_key),
        ]).form(&[
            ("j_username", username),
            ("j_password", &encrypted_password),
            ("j_checkcode", "请输入验证码"),
            ("op", "login"),
            ("spAuthChainCode", SP_AUTH_CHAIN_CODE),
            ("authnLcKey", authn_lc_key),
        ]).send().await?;
    debug!("login 3rd request: {:?}", response);
    debug!("login 3rd request header: {:?}", response.headers());
    // let text = response.text().await?;
    // if text.contains("增强认证") {
    //     debug!("Need to send sms");
    //     let response = client.post(SMS_URL)
    //         .header("Referer", format!("https://iam.tongji.edu.cn/idp/authcenter/ActionAuthChain?entityId={}&authnLcKey={}", entity_id, authn_lc_key))
    //         .form(&[
    //             ("j_username", username),
    //             ("type", "sms")
    //         ]).send().await?;
    //     if response.status().is_success() {
    //         info!("已发送验证码");
    //     } else {
    //         info!("发送验证码失败");
    //         return Err(Error::SMSCodeSendError(response.text().await?));
    //     }
    //     let mut sms_code = String::default();
    //     std::io::stdin().read_line(&mut sms_code)?;

    //     let response = client.post(LOGIN_URL)
    //         .header("Referer", format!("https://iam.tongji.edu.cn/idp/authcenter/ActionAuthChain?entityId={}&authnLcKey={}", entity_id, authn_lc_key))
    //         .query(&[("authnLcKey", authn_lc_key)])
    //         .form(&[
    //             ("j_username", username),
    //             ("type", "sms"),
    //             ("sms_checkcode", &sms_code),
    //             ("popViewException", "Pop2"),
    //             ("op", "login"),
    //             ("spAuthChainCode", SP_AUTH_CHAIN_CODE),
    //             ("j_checkcode", "请输入验证码"),
    //         ]).send().await;
    //     debug!("sms login request: {:?}", response);
    // }
    // get token, uid, ts from params
    let (mut token, mut uid, mut ts) = (None, None, None);
    response.url().query_pairs().for_each(|(key, value)| {
        let value = (*value).to_string();
        match &*key {
            "token" => token = Some(value),
            "uid" => uid = Some(value),
            "ts" => ts = Some(value),
            _ => ()
        }
    });
    let token = token.ok_or(Error::UrlParamsNotFoundError("token".to_string()))?;
    let uid = uid.ok_or(Error::UrlParamsNotFoundError("uid".to_string()))?;
    let ts = ts.ok_or(Error::UrlParamsNotFoundError("ts".to_string()))?;
    debug!("token: {}, uid: {}, ts: {}", token, uid, ts);

    let response = client.post(LOGIN_URL3)
        .header("Referer", format!("https://1.tongji.edu.cn/ssologin?token={}&uid={}&ts={}", token, uid, ts))
        .form(&[
            ("token", token),
            ("ts", ts),
            ("uid", uid)
        ]).send().await?.json::<serde_json::Value>().await?;
    let session_id = response.get("data")
        .and_then(|map| map.get("sessionid"))
        .and_then(|session_id| session_id.as_str())
        .and_then(|s| Some(s.to_string()));
    debug!("login result: session_id = {:?}", session_id);
    session_id.ok_or(Error::FieldNotFound("data.sessionid".to_string()))
}