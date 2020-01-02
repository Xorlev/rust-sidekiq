use std::default::Default;
use std::env;
use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Value;
use bb8_redis::{bb8, redis, RedisConnectionManager, RedisPool};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use serde_json;
use bb8::RunError;

const REDIS_URL_ENV: &str = "REDIS_URL";
const REDIS_URL_DEFAULT: &str = "redis://127.0.0.1/";

#[derive(Debug)]
pub struct ClientError {
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Redis(redis::RedisError),
    PoolTimeout,
}

pub async fn create_redis_pool() -> Result<RedisPool, ClientError> {
    let redis_url =
        &env::var(&REDIS_URL_ENV.to_owned()).unwrap_or_else(|_| REDIS_URL_DEFAULT.to_owned());
    let client = redis::Client::open(redis_url.as_str())?;
    let manager = RedisConnectionManager::new(client)?;
    let pool = bb8::Pool::builder().build(manager).await?;
    let redis_pool = RedisPool::new(pool);

    Ok(redis_pool)
}

pub struct Job {
    pub class: String,
    pub args: Vec<Value>,
    pub retry: i64,
    pub queue: String,
    pub jid: String,
    pub created_at: u64,
    pub enqueued_at: u64,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::Redis(ref err) => err.fmt(f),
            ErrorKind::PoolTimeout => f.write_str(self.description()),
        }
    }
}

impl Error for ClientError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::Redis(ref err) => err.description(),
            ErrorKind::PoolTimeout => "pool timeout",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self.kind {
            ErrorKind::Redis(ref err) => Some(err),
            ErrorKind::PoolTimeout => None,
        }
    }
}

impl From<redis::RedisError> for ClientError {
    fn from(error: redis::RedisError) -> ClientError {
        ClientError {
            kind: ErrorKind::Redis(error),
        }
    }
}

impl From<bb8::RunError<redis::RedisError>> for ClientError {
    fn from(error: bb8::RunError<redis::RedisError>) -> ClientError {
        let kind = match error {
            RunError::User(e) => ErrorKind::Redis(e),
            RunError::TimedOut => ErrorKind::PoolTimeout,
        };

        ClientError { kind }
    }
}

impl Default for JobOpts {
    fn default() -> JobOpts {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;
        let jid = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .collect::<String>();
        JobOpts {
            retry: 25,
            queue: "default".to_string(),
            jid,
            created_at: now,
            enqueued_at: now,
        }
    }
}

pub struct JobOpts {
    pub retry: i64,
    pub queue: String,
    pub jid: String,
    pub created_at: u64,
    pub enqueued_at: u64,
}

/// # Examples
///
/// ```
/// use std::default::Default;
/// use sidekiq::Value;
/// use sidekiq::{Job, JobOpts};
///
/// // Create a job
/// let class = "MyClass".to_string();
/// let job_opts = JobOpts {
///     queue: "test".to_string(),
///     ..Default::default()
/// };
/// let job = Job::new(class, vec![sidekiq::Value::Null], job_opts);
/// ```
impl Job {
    pub fn new(class: String, args: Vec<Value>, opts: JobOpts) -> Job {
        Job {
            class,
            args,
            retry: opts.retry,
            queue: opts.queue,
            jid: opts.jid,
            created_at: opts.created_at,
            enqueued_at: opts.enqueued_at,
        }
    }
}

impl Serialize for Job {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Job", 7)?;
        s.serialize_field("class", &self.class)?;
        s.serialize_field("args", &self.args)?;
        s.serialize_field("retry", &self.retry)?;
        s.serialize_field("queue", &self.queue)?;
        s.serialize_field("jid", &self.jid)?;
        s.serialize_field("created_at", &self.created_at)?;
        s.serialize_field("enqueued_at", &self.enqueued_at)?;
        s.end()
    }
}

pub struct ClientOpts {
    pub namespace: Option<String>,
}

impl Default for ClientOpts {
    fn default() -> ClientOpts {
        ClientOpts { namespace: None }
    }
}

pub struct Client {
    pub redis_pool: RedisPool,
    pub namespace: Option<String>,
}

/// # Examples
///
/// ```no_run
///
/// use sidekiq::{Job, Value};
/// use sidekiq::{Client, ClientOpts, create_redis_pool};
///
/// let ns = "test";
/// let client_opts = ClientOpts {
///     namespace: Some(ns.to_string()),
///     ..Default::default()
/// };
/// # futures::executor::block_on(async {
/// let pool = create_redis_pool().await.unwrap();
/// let client = Client::new(pool, client_opts);
/// let class = "MyClass".to_string();
/// let job = Job::new(class, vec![sidekiq::Value::Null], Default::default());
/// match client.push(job).await {
///     Ok(_) => {},
///     Err(err) => {
///         println!("Sidekiq push failed: {}", err);
///     },
/// }
/// # });
/// ```
impl Client {
    pub fn new(redis_pool: RedisPool, opts: ClientOpts) -> Client {
        Client {
            redis_pool,
            namespace: opts.namespace,
        }
    }

    pub async fn push(&self, job: Job) -> Result<(), ClientError> {
        self.raw_push(&[job]).await
    }

    pub async fn push_bulk(&self, jobs: &[Job]) -> Result<(), ClientError> {
        self.raw_push(jobs).await
    }

    async fn raw_push(&self, payloads: &[Job]) -> Result<(), ClientError> {
        let payload = &payloads[0];
        let to_push = payloads
            .iter()
            .map(|entry| serde_json::to_string(&entry).unwrap())
            .collect::<Vec<_>>();

        let mut conn = self.redis_pool.pool().get().await?;
        let connection = conn.as_mut().unwrap();
        redis::pipe()
            .atomic()
            .cmd("SADD")
            .arg("queues")
            .arg(payload.queue.to_string())
            .ignore()
            .cmd("LPUSH")
            .arg(self.queue_name(&payload.queue))
            .arg(to_push).query_async(connection).await?;

        Ok(())
    }

    fn queue_name(&self, queue: &str) -> String {
        if let Some(ref ns) = self.namespace {
            format!("{}:queue:{}", ns, queue)
        } else {
            format!("queue:{}", queue)
        }
    }
}
