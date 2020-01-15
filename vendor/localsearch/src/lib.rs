use radix_trie::Trie;

pub type Id = String;
pub type Text = String;

pub struct ResultItem {
    pub id: Id,
    pub text: Text,
    pub score: f64,
}

pub struct Document {
    pub id: Id,
    pub text: Text,
}

pub struct LocalSearch {
    db: Trie<Id, Text>
}

impl LocalSearch {
    pub fn new() -> Self {
        Self {
            db: Trie::new(),
        }
    }

    pub fn with_documents(documents: impl Iterator<Item=Document>) -> Self {
        let mut local_search = Self::new();
        local_search.add_documents(documents);
        local_search
    }

    pub fn add_documents(&mut self, documents: impl Iterator<Item=Document>) {
        for document in documents {
            self.add_document(document)
        }
    }

    pub fn add_document(&mut self, document: Document) {

    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<ResultItem> {
        vec![
            ResultItem {
                id: "dummy_id".into(),
                text: "dummy_text".into(),
                score: 1.,
            }
        ]
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
