use crate::error::LastFMError;
use crate::pairs;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    LastFMError(LastFMError), // attempt error first
    Valid(T),
}

impl<T> Response<T> {
    pub fn to_result(self) -> Result<T, LastFMError> {
        match self {
            Response::Valid(val) => Ok(val),
            Response::LastFMError(err) => Err(err),
        }
    }
}

pub struct ApiCall {
    params: HashMap<String, String>,
}

impl ApiCall {
    pub fn new(api_key: &str, method: &str, session_key: Option<&str>) -> Self {
        let mut params = HashMap::new();
        params.insert("api_key".to_string(), api_key.to_string());
        params.insert("method".to_string(), method.to_string());
        if let Some(session_key) = session_key {
            params.insert("sk".to_string(), session_key.to_string());
        }
        Self { params }
    }

    #[allow(dead_code)]
    pub fn params<'a>(self, pairs: impl Iterator<Item = (&'a str, &'a str)>) -> Self {
        let mut params = self.params;
        params.extend(pairs.map(|(key, value)| (key.to_string(), value.to_string())));
        Self { params, ..self }
    }

    pub fn struct_params<T>(self, msg: T) -> Result<Self, pairs::InvalidStructError>
    where
        T: serde::Serialize,
    {
        let pairs = pairs::to_pairs(msg)?;
        let mut params = self.params;
        params.extend(pairs.into_iter());
        Ok(Self { params, ..self })
    }

    fn signature(&self, secret: &[u8]) -> String {
        let mut params = self.params.iter().collect::<Vec<_>>();
        params.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut prehash = params.iter().fold(Vec::new(), |mut s, (k, v)| {
            s.extend(k.bytes());
            s.extend(v.bytes());
            s
        });
        prehash.extend(secret);
        format!("{:x}", md5::compute(prehash))
    }

    pub fn to_url(self, secret: &[u8], endpoint: &str, append_signature: bool) -> url::Url {
        let mut base = url::Url::parse(endpoint).unwrap();
        {
            let mut query = base.query_pairs_mut();
            let mut params = self.params.iter().collect::<Vec<_>>();
            params.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (key, value) in &params {
                query.append_pair(key, value);
            }
            if append_signature {
                let signature = self.signature(secret);
                query.append_pair("api_sig", &signature);
            }
            query.append_pair("format", "json");
        }
        #[cfg(debug_assertions)]
        println!("URL: {}", base);
        base
    }
}
