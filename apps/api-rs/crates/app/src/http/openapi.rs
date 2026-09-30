use axum::Router;
use regex::Regex;
use utoipa::OpenApi;
use utoipa::openapi::path::{Operation, ParameterBuilder, ParameterIn};
use utoipa::openapi::schema::{Object, ObjectBuilder, Type};
use utoipa::openapi::{ContentBuilder, Ref, Required, ResponseBuilder};
use utoipa_swagger_ui::SwaggerUi;

use super::error::Problem;

#[derive(OpenApi)]
#[openapi(
    info(title = "Clinicore API", version = "0.1.0"),
    components(schemas(Problem))
)]
struct ApiDoc;

pub fn document() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn matching(format: &Regex) -> Object {
    ObjectBuilder::new()
        .schema_type(Type::String)
        .pattern(Some(format.as_str()))
        .build()
}

pub fn document_guards(api: &mut utoipa::openapi::OpenApi) {
    for item in api.paths.paths.values_mut() {
        if let Some(operation) = item.get.as_mut() {
            document_guarded(operation, false);
        }
        for operation in [
            &mut item.post,
            &mut item.put,
            &mut item.patch,
            &mut item.delete,
        ]
        .into_iter()
        .flatten()
        {
            document_guarded(operation, true);
        }
    }
}

fn document_guarded(operation: &mut Operation, changes_state: bool) {
    let client = ParameterBuilder::new()
        .name("Clinicore-Client")
        .parameter_in(ParameterIn::Header)
        .required(Required::False)
        .description(Some(
            "mobile moves the session to the body and the Authorization header; absent keeps the cookies; any other value answers 400 invalid-client",
        ))
        .schema(Some(
            ObjectBuilder::new()
                .schema_type(Type::String)
                .enum_values(Some(["mobile"])),
        ))
        .build();
    operation
        .parameters
        .get_or_insert_with(Vec::new)
        .push(client.into());

    let mut problems = vec![
        (
            "400",
            "invalid-client, or validation-failed on a request body",
        ),
        ("429", "rate-limited"),
        ("503", "service-unavailable, with Redis out"),
    ];
    if changes_state {
        problems.push(("403", "invalid-origin"));
    }
    for (status, description) in problems {
        operation
            .responses
            .responses
            .entry(status.to_string())
            .or_insert_with(|| problem(description).into());
    }
}

fn problem(description: &str) -> utoipa::openapi::Response {
    ResponseBuilder::new()
        .description(description)
        .content(
            "application/problem+json",
            ContentBuilder::new()
                .schema(Some(Ref::from_schema_name("Problem")))
                .build(),
        )
        .build()
}

pub fn routes(api: utoipa::openapi::OpenApi) -> Router {
    SwaggerUi::new("/api").url("/api-json", api).into()
}
