use super::super::api_server::{ApiServer, CorsOptions};
use crate::{generator::oapi::generator::OpenApiGenerator, test_utils, CacheEndpoint};
use dozer_types::serde_json::{json, Value};

#[test]
fn test_generate_oapi() {
    let (schema, secondary_indexes) = test_utils::get_schema();
    let endpoint = test_utils::get_endpoint();

    let oapi_generator = OpenApiGenerator::new(
        schema,
        secondary_indexes,
        endpoint.name.to_owned(),
        endpoint,
        vec![format!("http://localhost:{}", "8080")],
    );
    let generated = oapi_generator.generate_oas3().unwrap();

    assert_eq!(generated.paths.paths.len(), 4, " paths must be generated");
}

#[actix_web::test]
async fn list_route() {
    let endpoint = test_utils::get_endpoint();
    let mut schema_name = endpoint.to_owned().path;
    schema_name.remove(0);
    let cache = test_utils::initialize_cache(&schema_name, None);
    let api_server = ApiServer::create_app_entry(
        None,
        CorsOptions::Permissive,
        vec![CacheEndpoint {
            cache,
            endpoint: endpoint.clone(),
        }],
    );
    let app = actix_web::test::init_service(api_server).await;

    let req = actix_web::test::TestRequest::get()
        .uri(&endpoint.path)
        .to_request();
    let res = actix_web::test::call_service(&app, req).await;
    assert!(res.status().is_success());

    let body: Value = actix_web::test::read_body_json(res).await;
    assert!(body.is_array(), "Must return an array");
    assert!(!body.as_array().unwrap().is_empty(), "Must return records");
}

#[actix_web::test]
async fn count_and_query_route() {
    let endpoint = test_utils::get_endpoint();
    let mut schema_name = endpoint.to_owned().path;
    schema_name.remove(0);
    let cache = test_utils::initialize_cache(&schema_name, None);
    let api_server = ApiServer::create_app_entry(
        None,
        CorsOptions::Permissive,
        vec![CacheEndpoint {
            cache,
            endpoint: endpoint.clone(),
        }],
    );
    let app = actix_web::test::init_service(api_server).await;

    let filter = json!({"$filter": {"film_id":  268}});
    let req = actix_web::test::TestRequest::post()
        .uri(&format!("{}/count", endpoint.path))
        .set_json(&filter)
        .to_request();
    let res = actix_web::test::call_service(&app, req).await;
    assert!(res.status().is_success());

    let body: Value = actix_web::test::read_body_json(res).await;
    assert_eq!(body.as_u64().unwrap(), 1);

    let req = actix_web::test::TestRequest::post()
        .uri(&format!("{}/query", endpoint.path))
        .set_json(&filter)
        .to_request();
    let res = actix_web::test::call_service(&app, req).await;
    assert!(res.status().is_success());

    let body: Value = actix_web::test::read_body_json(res).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[actix_web::test]
async fn get_route() {
    let endpoint = test_utils::get_endpoint();
    let mut schema_name = endpoint.to_owned().path;
    schema_name.remove(0);
    let cache = test_utils::initialize_cache(&schema_name, None);
    let api_server = ApiServer::create_app_entry(
        None,
        CorsOptions::Permissive,
        vec![CacheEndpoint {
            cache,
            endpoint: endpoint.clone(),
        }],
    );
    let app = actix_web::test::init_service(api_server).await;
    let req = actix_web::test::TestRequest::get()
        .uri(&format!("{}/{}", endpoint.path, 268))
        .to_request();

    let res = actix_web::test::call_service(&app, req).await;
    assert!(res.status().is_success());

    let body: Value = actix_web::test::read_body_json(res).await;
    assert!(body.is_object(), "Must return an object");
    let val = body.as_object().unwrap();
    assert_eq!(
        val.get("film_id").unwrap().to_string(),
        "268".to_string(),
        "Must be equal"
    );
}
