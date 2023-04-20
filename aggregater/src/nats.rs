//use async_nats::jetstream;
//use async_nats::jetstream::consumer::push::Messages;
//use async_nats::jetstream::consumer::DeliverPolicy;
use async_nats::jetstream::{stream::Config, consumer::push::Messages};
use async_nats::jetstream::Context;
use async_nats::Client;
use async_nats::jetstream::consumer::{AckPolicy, PushConsumer, DeliverPolicy};
//use async_nats::{Client, Message, Subscriber};

use bytes::Bytes;
use anyhow::{anyhow, Result};
use dashmap::DashSet;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use tokio_stream::StreamExt;
use std::marker::PhantomData;
use std::sync::Arc;


pub trait JetStreamable {
    /// Returns the name of the JetStream associated with this message type.
    fn subject(&self) -> String;
}

trait NatsResultExt<T> {
    fn to_anyhow(self) -> Result<T>;

    fn with_message(self, message: &'static str) -> Result<T>;
}

impl<T> NatsResultExt<T> for std::result::Result<T, async_nats::Error> {
    fn with_message(self, message: &'static str) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(anyhow!("NATS Error: {:?} ({})", err, message)),
        }
    }

    fn to_anyhow(self) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(anyhow!("NATS Error: {:?}", err)),
        }
    }
}

#[derive(Clone)]
pub struct TypedNats {
    nc: Client,
    jetstream: Context,
    jetstream_created_streams: Arc<DashSet<String>>,
    config: Config,
    deliver_subject: String,
    consumer_name: String,
}

impl TypedNats {
    #[must_use]
    pub fn new(nc: Client) -> Self {
        let jetstream = async_nats::jetstream::new(nc.clone());
        let stream_name = "stats".to_string();
        let subjects = vec!["stats.events.volume".to_string(), "stats.events.nexus".to_string(), "stats.events.pool".to_string(), "stats.events.replica".to_string()];
        let config = async_nats::jetstream::stream::Config {
                name: stream_name,
                subjects,
                max_messages: 100,
                ..async_nats::jetstream::stream::Config::default()
            };
        let deliver_subject = nc.new_inbox();
        let consumer_name = "stats-consumer".to_string();
        TypedNats {
            nc,
            jetstream,
            jetstream_created_streams: Arc::default(),
            config,
            deliver_subject,
            consumer_name
        }
    }

    pub async fn ensure_jetstream_exists(&self) -> Result<()> {
        if !self.jetstream_created_streams.contains("stats") {
            self.add_jetstream_stream().await?;

            self.jetstream_created_streams
                .insert("stats".to_string());
        }

        Ok(())
    }


    async fn add_jetstream_stream(&self) -> Result<()> {
        //tracing::debug!(name = config.name, "Creating jetstream stream.");
        self.jetstream
            .get_or_create_stream(self.config.clone())
            .await
            .to_anyhow()?;

        Ok(())
    }

    pub async fn subscribe_jetstream<T: Serialize + DeserializeOwned>(&self) -> Result<JetstreamSubscription<T>> {
        // An empty string is equivalent to no subject filter.
        //let subject = subject.map(|d| d.subject).unwrap_or_default();
        self.ensure_jetstream_exists().await?;
        //let stream_name = T::stream_name();

        let stream = self.jetstream.get_stream(self.config.name.clone()).await.to_anyhow()?;
        //let has_pending = stream.cached_info().state.messages > 0;
        //let deliver_subject = self.nc.new_inbox();

        let consumer = stream
            .get_or_create_consumer(self.consumer_name.as_str(), async_nats::jetstream::consumer::push::Config {
                durable_name: Some(self.consumer_name.clone()),
                deliver_policy: DeliverPolicy::All,
                deliver_subject: self.deliver_subject.clone(),
                max_ack_pending: 1, // NOTE: If you remove this or change the value,
                // the resultant stream is no longer guaranteed to be in order, and call sites
                // that rely on ordered messages will break, nondeterministically
                ..Default::default()
            })
            .await
            .to_anyhow()?;

        let stream: Messages = consumer.messages().await.to_anyhow()?;

        Ok(JetstreamSubscription {
            stream,
            _ph: PhantomData::default(),
        })
    }
}

pub struct JetstreamSubscription<T> {
    stream: Messages,
    _ph: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> JetstreamSubscription<T> {
    pub async fn next(&mut self) -> Option<T> {
        loop {
            if let Some(message) = self.stream.next().await {
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::error!(?error, "Error accessing jetstream message.");
                        continue;
                    }
                };
                message
                    .ack()
                    .await.unwrap();
                let value: Result<T, _> = serde_json::from_slice(&message.payload);
                match value {
                    Ok(value) => return Some(value),
                    Err(_error) => {
                        println!("Error parsing jetstream message; message ignored.");
                    }
                }
            } else {
                return None;
            }
        }
    }
}