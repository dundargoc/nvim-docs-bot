use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent, TextMessageEventContent,
    },
    ruma::user_id,
    Client,
};

use std::process::exit;

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // parse the command line for password
    let password = match env::args().nth(1) {
        Some(a) => a,
        _ => {
            eprintln!("Usage: {} <password>", env::args().next().unwrap());
            exit(1)
        }
    };

    let username = user_id!("@nvim-bot:matrix.org");
    let client = Client::builder().server_name(username.server_name()).build().await?;

    client.login_username(username, &password).send().await?;

    client.sync_once(SyncSettings::default()).await.unwrap();

    // now that we've synced, let's attach a handler for incoming room messages, so
    // we can react on it
    client.add_event_handler(on_room_message);

    // since we called `sync_once` before we entered our sync loop we must pass
    // that sync token to `sync`
    let settings = SyncSettings::default().token(client.sync_token().await.unwrap());

    // Syncing is important to synchronize the client state with the server.
    // This method will never return.
    client.sync(settings).await?; // this essentially loops until we kill the bot

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
    if let Room::Joined(room) = room {
        let msg_body = match event.content.msgtype {
            MessageType::Text(TextMessageEventContent { body, .. }) => body,
            _ => return,
        };

        // here comes the actual "logic": when the bot see's a `!party` in the message,
        // it responds
        if msg_body.contains("!h") || msg_body.contains(":h") {
            let content = RoomMessageEventContent::text_plain(
                "Help requested, but I can't do anything at the moment.",
            );

            // send our message to the room we found the "!party" command in
            // the last parameter is an optional transaction id which we don't
            // care about.
            room.send(content, None).await.unwrap();
        }
    }
}
