use actix_web::{
    dev::ServiceRequest,
    web::{self, ReqData},
    Error, HttpMessage, HttpRequest, HttpResponse,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use dozer_types::{
    models::api_security::ApiSecurity,
    serde_json::{json, Value},
};

use crate::errors::{ApiError, AuthError};

use super::{Access, Authorizer};

pub async fn auth_route(
    access: Option<ReqData<Access>>,
    req: HttpRequest,
    tenant_access: web::Json<Value>,
) -> Result<HttpResponse, ApiError> {
    let access = match access {
        Some(access) => access.into_inner(),
        None => Access::All,
    };

    let tenant_access = dozer_types::serde_json::from_value(tenant_access.0)
        .map_err(ApiError::map_deserialization_error)?;
    match access {
        // Master Key or Uninitialized
        Access::All => {
            let secret = get_secret(&req)?;
            let auth = Authorizer::new(secret, None, None);
            let token = auth.generate_token(tenant_access, None).unwrap();
            Ok(HttpResponse::Ok().body(json!({ "token": token }).to_string()))
        }
        Access::Custom(_) => Err(ApiError::ApiAuthError(AuthError::Unauthorized)),
    }
}

fn get_secret(req: &HttpRequest) -> Result<&str, AuthError> {
    let api_security = req
        .app_data::<ApiSecurity>()
        .ok_or(AuthError::Unauthorized)?;

    match api_security {
        ApiSecurity::Jwt(secret) => Ok(secret.as_str()),
    }
}
pub async fn validate(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let api_security = req
        .app_data::<ApiSecurity>()
        .expect("We only validate bearer tokens if ApiSecurity is set");
    match api_security {
        ApiSecurity::Jwt(secret) => {
            let api_auth = Authorizer::new(secret, None, None);
            let res = api_auth
                .validate_token(credentials.token())
                .map_err(|e| (Error::from(ApiError::ApiAuthError(e))));

            match res {
                Ok(claims) => {
                    // Provide access to all
                    req.extensions_mut().insert(claims.access);
                    Ok(req)
                }
                Err(e) => Err((e, req)),
            }
        }
    }
}
