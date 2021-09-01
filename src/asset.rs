use relm4::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Fetcher<T>
where
    T: FetcherExt,
{
    cache: HashMap<String, FetcherCache<T>>,
}

#[async_trait]
pub trait FetcherExt: Clone + std::fmt::Debug {
    async fn from_response(response: Option<surf::Response>) -> Self;
}

impl<T> Fetcher<T>
where
    T: FetcherExt,
{
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cache = HashMap::new();
    }

    pub async fn fetch(&mut self, url: String) -> T {
        // self.cache
        //     .entry(url.clone())
        //     .or_insert(FetcherCache::Queued);

        // println!("{:#?}", &self.cache);

        T::from_response(surf::get(url.clone()).await.ok()).await

        // if let Some(FetcherCache::Queued) = self.cache.get(&url) {
        //     println!("fetch: {}", &url);

        //     match surf::get(url.clone()).await {
        //         Ok(response) => {
        //             let future = T::from_response(Some(response));
        //             let result = future.await;

        //             self.cache
        //                 .insert(url.clone(), FetcherCache::Fetched(result))
        //         }
        //         Err(_) => self.cache.insert(url.clone(), FetcherCache::Failed),
        //     };
        // }

        // if let Some(FetcherCache::Fetched(result)) = self.cache.get(&url) {
        //     return Some(result.clone());
        // }

        // None
    }
}

#[derive(Debug)]
pub enum FetcherCache<T>
where
    T: FetcherExt,
{
    // (Box<surf::Response>)
    Queued,
    Failed,
    Fetched(T),
}
