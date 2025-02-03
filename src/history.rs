use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use ulid::Ulid;

use crate::chat::ChatOutput;

#[derive(Clone, Serialize, Deserialize)]
pub struct SavedChat<T> {
    pub ulid: Ulid,
    pub model: String,
    pub content: Vec<Party<T>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Party<T> {
    Query(String),
    Reply(T),
}

pub fn read_history(path: &Path) -> Vec<SavedChat<String>> {
    let path = path.to_path_buf().join("history.json");

    let Ok(file) = std::fs::File::open(&path) else {
        return vec![];
    };

    let Ok(v) = serde_json::from_reader(&file) else {
        return vec![];
    };

    v
}

pub fn serialize_history(chats: &[SavedChat<String>]) -> String {
    serde_json::to_string_pretty(chats).unwrap()
}

pub async fn write_history(path: PathBuf, chats: String) -> std::io::Result<()> {
    let path = path.join("history.json");
    let tmp_path = path.clone().with_extension(".json.tmp");

    let mut file = tokio::fs::File::create(&tmp_path).await?;
    file.write_all(chats.as_bytes()).await?;
    std::fs::rename(tmp_path, path)?;
    Ok(())
}

impl SavedChat<String> {
    pub fn to_chat_output(self) -> SavedChat<ChatOutput> {
        let content = self
            .content
            .into_iter()
            .map(|p| match p {
                Party::Query(q) => Party::Query(q),
                Party::Reply(s) => {
                    let mut chat_output = ChatOutput::new();
                    chat_output.add_content(&s);
                    Party::Reply(chat_output)
                }
            })
            .collect::<Vec<_>>();
        SavedChat {
            ulid: self.ulid,
            model: self.model,
            content,
        }
    }

    pub fn description(&self) -> String {
        if self.content.is_empty() {
            String::new()
        } else {
            match &self.content[0] {
                Party::Query(p) => p.chars().take(40).collect::<String>(),
                Party::Reply(_) => String::new(),
            }
        }
    }
}

impl SavedChat<ChatOutput> {
    pub fn flatten_output(self) -> SavedChat<String> {
        let content = self
            .content
            .into_iter()
            .map(|p| match p {
                Party::Query(q) => Party::Query(q),
                Party::Reply(s) => Party::Reply(s.raw()),
            })
            .collect::<Vec<_>>();
        SavedChat {
            ulid: self.ulid,
            model: self.model,
            content,
        }
    }
}
