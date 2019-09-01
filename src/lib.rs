use std::{fmt, io};

use async_std::fs::File;
use async_std::prelude::*;

use failure_derive::Fail;

use futures::compat::Future01CompatExt;

use serde::de::DeserializeOwned;

use surf::{middleware::HttpClient, Client, Request};

mod photos;
pub use crate::photos::{Photo, Post, Posts};

mod macros;

mod uri;
use crate::uri::{QueryParameters, UriPath};

mod response;
use crate::response::{Response, TotalPosts};

const MAX_PAGE_SIZE: &'static str = "20";

type SurfError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "api error: {}", _0)]
    Api(String),

    #[fail(display = "json error: {}", _0)]
    Json(serde_json::error::Error),

    #[fail(display = "io error: {}", _0)]
    Io(io::Error),

    #[fail(display = "surf error: {}", _0)]
    Surf(SurfError),
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Error {
        Error::Json(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<SurfError> for Error {
    fn from(error: SurfError) -> Error {
        Error::Surf(error)
    }
}

pub struct Blog<C, E>
where
    C: surf::middleware::HttpClient<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    client: Client<C>,
    api_key: String,
    blog_identifier: String,
}

impl<C, E> Blog<C, E>
where
    C: surf::middleware::HttpClient<Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn new(client: Client<C>, api_key: String, blog_identifier: String) -> Blog<C, E> {
        Blog {
            client,
            api_key,
            blog_identifier,
        }
    }

    async fn tumblr_get<'a, 'de, T>(
        &self,
        path: UriPath,
        params: QueryParameters<'a>,
    ) -> Result<Response<T>, Error>
    where
        T: 'static + Clone + fmt::Debug + DeserializeOwned,
    {
        let uri = uri::tumblr_uri(&self.blog_identifier, &path, &params);
        let v: Response<T> = self.client.get(uri).recv_json().await?;

        if v.meta.is_success() {
            Ok(v)
        } else {
            Err(Error::Api(v.meta.msg.clone()))
        }
    }

    pub async fn fetch_post_count(&self) -> Result<usize, Error> {
        let path = uri_path![posts / photo];
        let params = uri_params! { api_key => &self.api_key, limit => "1" };

        let v: Response<TotalPosts> = self.tumblr_get::<TotalPosts>(path, params).await?;
        Ok(v.response.amount)
    }

    pub async fn fetch_posts_page(&self, page_start_index: usize) -> Result<Vec<Post>, Error> {
        let path = uri_path![posts / photo];
        let params = uri_params! {
            api_key => &self.api_key,
            limit => MAX_PAGE_SIZE,
            offset => format!("{}", page_start_index)
        };

        let v: Response<Posts> = self.tumblr_get::<Posts>(path, params).await?;
        Ok(v.response.posts)
    }

    pub async fn download_file(&self, post: Post) -> Result<(), Error> {
        dbg!(post);
        let mut file = File::create("lol.jpg").await?;
        file.write_all(b"something").await?;
        Ok(())
    }
}
