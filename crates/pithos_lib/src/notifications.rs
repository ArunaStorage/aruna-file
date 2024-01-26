use std::sync::RwLock;

use crate::{structs::FileContext, transformer::TransformerType};
use async_channel::Sender;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum HashType {
    Sha1,
    Md5,
    Other(String),
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Message {
    Completed,
    Finished,
    FileContext(FileContext),
    Hash((HashType, String)),
    Metadata((Option<Vec<u8>>, String)), // Optional different Key, JSON Metadata value
    SizeInfo(u64),
    Compression(bool),
    EditList(Vec<u64>),
    ShouldFlush,
    Skip,
    Custom((String, Vec<u8>)),
}

pub struct Notifier {
    read_writer: Sender<Message>,
    notifiers: RwLock<Vec<(TransformerType, Sender<Message>)>>,
}

impl Notifier {
    pub fn new(read_writer: Sender<Message>) -> Self {
        Self {
            read_writer,
            notifiers: RwLock::new(Vec::new()),
        }
    }

    pub fn add_transformer(&self, trans: (TransformerType, Sender<Message>)) {
        self.notifiers.write().unwrap().push(trans);
    }

    pub fn send_next(&self, idx: usize, message: Message) -> anyhow::Result<()> {
        if idx + 1 < self.notifiers.read().unwrap().len() {
            self.notifiers.read().unwrap()[idx + 1]
                .1
                .try_send(message)?;
        }
        Ok(())
    }

    pub fn send_first(&self, message: Message) -> anyhow::Result<()> {
        if let Some((_, sender)) = self.notifiers.read().unwrap().first() {
            sender.try_send(message)?;
        }
        Ok(())
    }

    pub fn send_next_type(
        &self,
        idx: usize,
        trans_type: TransformerType,
        message: Message,
    ) -> anyhow::Result<()> {
        for (trans, sender) in self.notifiers.read().unwrap()[idx..]
            .iter()
            .chain(self.notifiers.read().unwrap().iter())
        {
            if trans == &trans_type {
                sender.try_send(message)?;
                break;
            }
        }
        Ok(())
    }

    pub fn send_all_type(
        &self,
        trans_type: TransformerType,
        message: Message,
    ) -> anyhow::Result<()> {
        for (trans, sender) in self.notifiers.read().unwrap().iter() {
            if trans == &trans_type {
                sender.try_send(message.clone())?;
            }
        }
        Ok(())
    }

    pub fn send_all(&self, message: Message) -> anyhow::Result<()> {
        for (_, sender) in self.notifiers.read().unwrap().iter() {
            sender.try_send(message.clone())?;
        }
        Ok(())
    }

    pub fn send_read_writer(&self, message: Message) -> anyhow::Result<()> {
        self.read_writer.try_send(message)?;
        Ok(())
    }
}
