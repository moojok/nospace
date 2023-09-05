use ::error::AppResult;
use chrono::Utc;
use entity::{users::ActiveModel, ActiveValue};
use serde::{Deserialize, Serialize};
use util::{
    password::hash,
    validation::{validate_otp, validate_password},
};
use validr::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: Option<String>,
    pub password: Option<String>,
    pub secret: Option<String>,
    pub token: Option<String>,
    pub encrypted_private_key: Option<String>,
    pub pubkey: Option<String>,
}

impl Validation for CreateUser {
    fn rules(&self) -> Vec<Rule<Self>> {
        vec![
            rule_required!(email),
            rule_email!(email),
            rule_required!(password),
            rule_required!(pubkey),
            Rule::new("password", |obj: &Self, error| {
                if let Some(v) = &obj.password {
                    if !validate_password(v) {
                        error.add("weak_password");
                    }
                }
            }),
            Rule::new("secret", |obj: &Self, error| {
                if let Some(v) = &obj.secret {
                    if !validate_otp(v, obj.token.as_ref()) {
                        error.add("invalid_otp_token");
                    }
                }
            }),
            Rule::new("pubkey", |obj: &Self, error| {
                if let Some(v) = &obj.pubkey {
                    if cryptfns::public::from_str(v).is_err() {
                        error.add("invalid_pubkey_not_pkcs8_pem");
                    }
                }
            }),
        ]
    }

    fn modifiers(&self) -> Vec<Modifier<Self>> {
        vec![modifier_trim!(email), modifier_lowercase!(email)]
    }
}

impl CreateUser {
    pub fn into_active_model(self) -> AppResult<ActiveModel> {
        let data = self.validate()?;

        Ok(ActiveModel {
            id: ActiveValue::NotSet,
            email: ActiveValue::Set(data.email.unwrap()),
            password: ActiveValue::Set(data.password.map(hash)),
            secret: ActiveValue::Set(data.secret),
            pubkey: ActiveValue::Set(data.pubkey.unwrap()),
            encrypted_private_key: ActiveValue::Set(data.encrypted_private_key),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            updated_at: ActiveValue::Set(Utc::now().naive_utc()),
        })
    }
}
