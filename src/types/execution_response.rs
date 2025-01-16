pub enum ExecResponse {
    Simple(Vec<u8>),
    Multi(Vec<Vec<u8>>),
}

impl From<String> for ExecResponse {
    fn from(value: String) -> Self {
        Self::Simple(value.into_bytes())
    }
}

impl From<&String> for ExecResponse {
    fn from(value: &String) -> Self {
        Self::Simple(value.clone().into_bytes())
    }
}

impl From<Vec<u8>> for ExecResponse {
    fn from(value: Vec<u8>) -> Self {
        Self::Simple(value)
    }
}

impl From<&Vec<u8>> for ExecResponse {
    fn from(value: &Vec<u8>) -> Self {
        Self::Simple(value.clone())
    }
}

impl From<Vec<String>> for ExecResponse {
    fn from(value: Vec<String>) -> Self {
        let res: Vec<Vec<u8>> = value.iter().map(|i| i.clone().into_bytes()).collect();
        Self::Multi(res)
    }
}

impl From<Vec<Vec<u8>>> for ExecResponse {
    fn from(value: Vec<Vec<u8>>) -> Self {
        Self::Multi(value)
    }
}

impl IntoIterator for ExecResponse {
    type Item = Vec<u8>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ExecResponse::Simple(v) => vec![v].into_iter(),
            ExecResponse::Multi(vv) => vv.into_iter(),
        }
    }
}
