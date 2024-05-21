use serde_json::json as s_json;
use serenity::builder::*;
use serenity::interactions_endpoint::Verifier;
use serenity::json;
use serenity::model::application::*;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

// type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    let verifier = Verifier::new(std::env::var("DISCORD_PUBLIC_KEY")?.as_str());
    Ok(handle_request(req, &verifier)?)
}

fn handle_command(interaction: CommandInteraction) -> CreateInteractionResponse {
    CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(format!(
        "*Hello* from interactions webhook HTTP server! <@{}>\nThank you!",
        interaction.user.id
    )))
}

fn handle_request(
    request: Request,
    //  body: &mut Vec<u8>,
    verifier: &Verifier,
) -> Result<Response<Body>, Error> {
    let body = request.body();

    let find_header = |name| match request.headers().get(name) {
        Some(value) => value.to_str().unwrap_or(""),
        None => "",
    };

    let signature = find_header("X-Signature-Ed25519");
    let timestamp = find_header("X-Signature-Timestamp");

    eprintln!("signature: {signature}, timestamp: {timestamp}");

    if verifier.verify(signature, timestamp, &body).is_err() {
        eprintln!("Failed to verifier");
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .body(().into())?);
    }

    // Build Discord response
    let response = match json::from_slice::<Interaction>(body)? {
        // Discord rejects the interation endpoints URL if pings are not acknowleded
        // https://discord.com/developers/docs/tutorials/upgrading-to-application-commands#adding-an-interactions-endpoint-url
        Interaction::Ping(_) => CreateInteractionResponse::Pong,
        Interaction::Command(interaction) => handle_command(interaction),
        _ => return Ok(Response::builder().body(().into())?),
    };
    // eprintln!("{:?}", response);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(s_json!(&response).to_string().into())?)
}
