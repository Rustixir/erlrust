


#[derive(Clone, Debug)]
pub struct Url {
    data: String
}

impl Url {

    pub fn new(url: String) -> Self {
        Url { data: url }
    }

    pub fn get_url(&self) -> &String {
        return &self.data
    }
}

