//! [Sidekiq](https://github.com/mperham/sidekiq) client allowing to push jobs.
//! Using the [Sidekiq job
//! format](https://github.com/mperham/sidekiq/wiki/Job-Format) as reference.
//!
//! # Default environment variables
//!
//! `REDIS_URL`="redis://127.0.0.1/"
//!
#![doc(html_root_url = "https://docs.rs/sidekiq/0.8.6")]
#![deny(warnings)]
#![crate_name = "sidekiq"]

mod sidekiq;
pub use crate::sidekiq::{
    create_redis_pool, Client, ClientOpts, ClientError, Job, JobOpts
};
pub use serde_json::value::Value;
