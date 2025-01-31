use std::net::SocketAddr;

use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage},
    Ollama,
};

pub use ollama_rs::generation::chat::ChatMessageResponse;

#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
}

pub const DEFAULT_PORT: u16 = 11434;

impl OllamaConfig {
    pub fn localhost(port: u16) -> Self {
        Self {
            host: "localhost".to_string(),
            port,
        }
    }

    pub async fn tcp_connect(&self) -> std::io::Result<tokio::net::TcpStream> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse().unwrap();
        tokio::net::TcpStream::connect(addr).await
    }

    pub fn instance(&self) -> Ollama {
        Ollama::new(format!("http://{}", self.host), self.port)
    }
}

pub struct ChatMessageResponseStream(pub ollama_rs::generation::chat::ChatMessageResponseStream);

impl std::fmt::Debug for ChatMessageResponseStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChatMessageResponseStream")
    }
}

impl Clone for ChatMessageResponseStream {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct LocalModel(ollama_rs::models::LocalModel);

impl PartialEq for LocalModel {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
            && self.0.modified_at == other.0.modified_at
            && self.0.size == other.0.size
    }
}

impl std::fmt::Display for LocalModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}

impl Eq for LocalModel {}

pub struct ConnectionFailed;

pub async fn get_model_lists(api: &Ollama) -> Result<Vec<LocalModel>, ConnectionFailed> {
    api.list_local_models()
        .await
        .map(|v| v.into_iter().map(LocalModel).collect())
        .map_err(|_| ConnectionFailed)
}

pub async fn chat_stream(api: Ollama, prompt: String) -> ChatMessageResponseStream {
    let stream = api
        .send_chat_messages_stream(ChatMessageRequest::new(
            "deepseek-r1:32b".to_string(),
            vec![ChatMessage::user(prompt)],
        ))
        .await
        .unwrap();
    ChatMessageResponseStream(stream)
}
