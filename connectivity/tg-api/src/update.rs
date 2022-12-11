use serde::Deserialize;
use telegram_bot_raw::{
    CallbackQuery, ChannelPost, ChatMember, ExportChatInviteLink, InlineQuery, Integer, Message,
    MessageChat, Poll, PollAnswer, User, RawChat,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct Update {
    /// The update‘s unique identifier. Update identifiers start from a certain
    /// positive number and increase sequentially.
    #[serde(rename = "update_id")]
    pub id: Integer,

    /// Optional. New incoming message of any kind - text, photo, sticker, etc.
    pub message: Option<Message>,
    /// New version of a message that is known to the bot and was edited
    pub edited_message: Option<Message>,
    /// New incoming channel post of any kind — text, photo, sticker, etc.
    pub channel_post: Option<ChannelPost>,
    /// New version of a channel post that is known to the bot and was edited
    pub edited_channel_post: Option<ChannelPost>,
    pub inline_query: Option<InlineQuery>, //  ChosenInlineResult(ChosenInlineResult),
    pub callback_query: Option<CallbackQuery>,
    /// New poll state. Bots receive only updates about stopped polls and polls, which are sent by the bot
    pub poll: Option<Poll>,
    /// A user changed their answer in a non-anonymous poll. Bots receive new votes only in polls that were sent by the bot itself
    pub poll_answer: Option<PollAnswer>,

    pub chat_member: Option<ChatMemberUpdated>,
    pub my_chat_member: Option<ChatMemberUpdated>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct ChatMemberUpdated {
    pub date: Integer,
    pub chat: RawChat,
    pub from: User,
    pub old_chat_member: ChatMember,
    pub new_chat_member: ChatMember,
    pub chat_invite_link: Option<ChatInviteLink>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct ChatInviteLink {
    /// The invite link. If the link was created by another chat administrator, then the second part of the link will be replaced with “…”.
    invite_link: String,
    /// Creator of the link
    creator: User,
    /// True, if users joining the chat via the link need to be approved by chat administrators
    creates_join_request: bool,
    /// True, if the link is primary
    is_primary: bool,
    /// True, if the link is revoked
    is_revoked: bool,
    /// Optional. Invite link name
    name: String,
    /// Optional. Point in time (Unix timestamp) when the link will expire or has been expired
    expire_date: Integer,
    ///Optional. The maximum number of users that can be members of the chat simultaneously after joining the chat via this invite link; 1-99999
    member_limit: Integer,
    /// Optional. Number of pending join requests created using this link
    pending_join_request_count: Integer,
}
