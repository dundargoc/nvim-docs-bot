#![warn(clippy::pedantic)]

use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
    },
    ruma::user_id,
    Client,
};
use std::collections::HashMap;
use std::env;
use std::process::exit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse the command line for the password file
    let Some(password_file) = env::args().nth(1) else {
        eprintln!("Usage: {} <password-file>", env::args().next().unwrap());
        exit(1)
    };

    let binding = tokio::fs::read_to_string(password_file).await.unwrap();
    let password = binding.trim();

    let username = user_id!("@nvim-bot:matrix.org");
    let client = Client::builder()
        .server_name(username.server_name())
        .build()
        .await?;

    client.login_username(username, password).send().await?;

    client.sync_once(SyncSettings::default()).await.unwrap();

    // now that we've synced, let's attach a handler for incoming room messages, so
    // we can react on it
    client.add_event_handler(on_room_message);

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());

    // Syncing is important to synchronize the client state with the server.
    // This method will never return.
    Box::pin(client.sync(settings)).await?; // this essentially loops until we kill the bot

    Ok(())
}

// This fn is called whenever we see a new room message event. You notice that
// the difference between this and the other function that we've given to the
// handler lies only in their input parameters. However, that is enough for the
// rust-sdk to figure out which one to call one and only do so, when
// the parameters are available.
async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    // First, we need to unpack the message: We only want messages from rooms we are
    // still in and that are regular text messages - ignoring everything else.
    let Room::Joined(room) = room else {
        return;
    };

    let MessageType::Text(TextMessageEventContent { body: msg, .. }) = event.content.msgtype else {
        return;
    };

    // Add space after the "h" to prevent message like "!hello" to trigger
    let trigger = "!h ";
    let Some(msg) = msg.strip_prefix(trigger) else {
        return;
    };

    if room.name() != Some("nvim-bot-test".to_string()) {
        room.send(RoomMessageEventContent::text_plain("Not here dumbass, do it in the test room: https://matrix.to/#/#nvim-bot-test:matrix.org"), None)
        .await
        .unwrap();

        return;
    };

    let mut vec = Vec::new();

    let mut tags = HashMap::new();
    let text = tokio::fs::read_to_string("src/tags").await.unwrap();
    for line in text.lines() {
        let line_split = line.split_whitespace().collect::<Vec<&str>>();
        let tag = line_split[0];
        vec.push(tag);
        let file = line_split[1].replace(".txt", "");
        tags.insert(tag, file);
    }

    let msg = ex_help(msg).await;

    match msg {
        "" => "help.txt",
        msg => msg,
    };

    let tag = msg;
    let index = match vec.binary_search(&tag) {
        Ok(index) | Err(index) => index,
    };
    let tag = vec[index];
    let message = if let Some(file) = tags.get(tag) {
        format!("https://neovim.io/doc/user/{file}.html#{tag}")
    } else {
        format!("No help found for {tag}!")
    };
    room.send(RoomMessageEventContent::text_plain(message), None)
        .await
        .unwrap();
}

async fn ex_help(msg: &str) -> &str {
    // remove trailing blanks
    let msg = msg.trim();
    msg
}
