use juniper::{
    tests::fixtures::starwars::schema::{Database, Query},
    EmptyMutation, EmptySubscription, RootNode,
};
use rocket::{response::content, State};

type Schema = RootNode<'static, Query, EmptyMutation<Database>, EmptySubscription<Database>>;

#[rocket::get("/")]
fn graphiql() -> content::Html<String> {
    juniper_rocket_async::graphiql_source("/graphql")
}

#[rocket::get("/graphql?<request>")]
async fn get_graphql_handler(
    context: State<'_, Database>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: State<'_, Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute(&schema, &context).await
}

#[rocket::post("/graphql", data = "<request>")]
async fn post_graphql_handler(
    context: State<'_, Database>,
    request: juniper_rocket_async::GraphQLRequest,
    schema: State<'_, Schema>,
) -> juniper_rocket_async::GraphQLResponse {
    request.execute(&schema, &context).await
}

#[rocket::main]
async fn main() {
    rocket::ignite()
        .manage(Database::new())
        .manage(Schema::new(
            Query,
            EmptyMutation::<Database>::new(),
            EmptySubscription::<Database>::new(),
        ))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch()
        .await
        .expect("server to launch");
}