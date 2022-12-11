use telegram_bot_raw::{GetUpdates, SendMessage, UpdateKind};
use tg_api::Api;



async fn runner () {
    let token = std::env::var("BOT_KEY").unwrap();
    let api = Api::new(token);
    let me = api.get_updates(GetUpdates::new()).await;


    println!("me:  {:#?}", me);

    let first = me.first().unwrap();

    /*
    match &first.kind {
        UpdateKind::Message(msg) => {
            let chat = msg.chat.clone();
            let msg = SendMessage::new(chat, "I am bot!");
            if let Some(msg) = api.send_message(msg).await {
                println!("Just sent message: {:#?}", msg);
                
            }
        }
        _ => {println!("Unexpected");}
    }
    */
    


}

#[tokio::main]
async fn main() {
    runner().await
}
