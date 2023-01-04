use core::fmt;
use std::{fmt::Debug, path::PathBuf};

use app::{BoxFuture, FutureExt, Sink, SinkExt, Stream, StreamExt};
use telegram_bot_raw::{
    ChatId, ChatMemberStatus, ChatRef, GetUpdates, ParseMode::MarkdownV2, SendMessage, UpdateKind,
};
use tg_api::{Api, update::Update};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};
use tracing::{error, info, warn};

use crate::TelegramReporter;

#[derive(Debug)]
pub enum ChatMessage {
    Add(i64),
    Delete(i64),
}

impl TelegramReporter {
    pub fn send_loop<'f, St, T>(
        &'f self,
        mut rx: St,
        mut state_rx: impl Stream<Item = Vec<T>> + Send + Sync + Unpin + 'f,
    ) -> BoxFuture<'f, ()>
    where
        St: Stream<Item = ChatMessage> + Unpin + Send + 'f,
        T: fmt::Display + Sync + Send + 'f,
    {
        async move {
            let storage_path = self.storage_path.clone().into();
            let mut chats = match Self::load_chats(&storage_path).await {
                Ok(chats) => chats,
                Err(err) => {
                    warn!(?storage_path, "Cannot load file - will create new one");
                    Vec::new()
                }
            };
            info!(?chats, "Loaded chats");
            let api = self.api.clone();
            loop {
                select! {
                    chat = rx.next() => {
                        match chat {
                            Some(ChatMessage::Add(id)) => {
                                info!(?id, "Add chat");
                                chats.push(id);
                            }
                            Some(ChatMessage::Delete(id)) => {
                                if let Some(pos) = chats.iter().position(|i| *i == id) {
                                    info!(?id, "Remove chat");
                                    chats.remove(pos);
                                }
                            }
                            None => {break;}
                        }
                        info!(?chats, "Save chats to storage");
                        Self::save_chats(&chats, &storage_path).await;
                    },
                    signals = state_rx.next() => {
                        match signals {
                            Some(signals) => {
                                Self::broadcast(&api, &signals.into_iter().map(|s| format!("{}", s)).collect::<Vec<_>>(), &chats).await;
                            },
                            None => {break;}
                        }

                    }
                }
            }
        }
        .boxed()
    }

    async fn save_chats(chats: &Vec<i64>, path: &PathBuf) {
        let mut file = tokio::fs::File::create(path).await.unwrap();
        file.write_all(serde_json::to_string(chats).unwrap().as_bytes())
            .await
            .unwrap();
    }

    async fn load_chats(path: &PathBuf) -> Result<Vec<i64>, std::io::Error> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buf = Vec::new();
        let _total_size = file.read_to_end(&mut buf).await?;
        Ok(serde_json::from_slice(&buf)?)
    }

    async fn broadcast(api: &Api, text: &[String], chats: &[i64]) {
        let message = text.join("\n---------\n");
        if !message.is_empty() {
            for chat_id in chats {
                let chat = ChatRef::from_chat_id(ChatId::new(*chat_id));
                let mut send_message = SendMessage::new(chat, &message);
                send_message.parse_mode(MarkdownV2);

                api.send_message(send_message).await.unwrap();
            }
        }
    }

    async fn process_updates<S>(latest_update: &mut Option<i64>, updates: Vec<Update>, tx: &mut S) 

    where
        S: Sink<ChatMessage> + fmt::Debug + Unpin + Send + Sync,
        <S as app::Sink<ChatMessage>>::Error: fmt::Debug,
    {
        *latest_update = updates.last().map(|upd| upd.id);

        for update in updates {
            info!(?update, "Got update");
            if let Some(message) = update.message {
                info!(?message, "Got message");
                // Here comes code to store clients
                tx.send(ChatMessage::Add(message.chat.id().into()))
                    .await
                    .unwrap();
            }

            if let Some(chat_member) = update.my_chat_member {
                info!(?chat_member, "Got my chat member update");
                if matches!(chat_member.new_chat_member.status, ChatMemberStatus::Kicked) {
                    tx.send(ChatMessage::Delete(chat_member.chat.id))
                        .await
                        .unwrap();
                }
                // Here comes code to remove clients
            }

            if let Some(chat_member) = update.chat_member {
                info!(?chat_member, "Got chat member update");

                tx.send(ChatMessage::Delete(chat_member.chat.id))
                    .await
                    .unwrap();
                // Here comes code to remove clients
            }
        }
    }

    pub fn bot_loop<'f, S>(&'f self, mut tx: S) -> BoxFuture<'f, ()>
    where
        S: Sink<ChatMessage> + fmt::Debug + Unpin + Send + Sync + 'f,
        <S as app::Sink<ChatMessage>>::Error: fmt::Debug,
    {
        let api = self.api.clone();
        async move {
            let mut latest_update = None;
            loop {
                let request = latest_update
                    .take()
                    .map(|update_id| {
                        let mut updates = GetUpdates::new();
                        updates.offset(update_id + 1);
                        updates
                    })
                    .unwrap_or_else(GetUpdates::new);
                match api.get_updates(request).await {
                    Ok(updates) => Self::process_updates(&mut latest_update, updates, &mut tx).await,
                    Err(some_error) => {
                        error!("{some_error}");
                    }
                }

            }
        }
        .boxed()
    }
}
