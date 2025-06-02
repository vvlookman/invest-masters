use crate::{
    error::InvmstResult,
    llm::{ChatCompletionOptions, ChatCompletionStream, ChatMessage, Role},
};

pub mod open_ai;

pub trait ChatProvider {
    fn chat_completion(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
    ) -> impl std::future::Future<Output = InvmstResult<ChatMessage>> + Send;

    fn chat_completion_stream(
        &self,
        messages: &[ChatMessage],
        options: &ChatCompletionOptions,
    ) -> impl std::future::Future<Output = InvmstResult<ChatCompletionStream>> + Send;
}
