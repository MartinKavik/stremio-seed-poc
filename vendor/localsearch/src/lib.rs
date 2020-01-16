use std::{collections::HashMap, iter::FromIterator};
use radix_trie::Trie;
use radix_fmt::radix_36;
use unicode_segmentation::UnicodeSegmentation;

// @TODO new types?
pub type Id = String;
pub type Text = String;
type RawToken = str;
type Token = String;

#[derive(PartialEq, Eq, Hash, Clone)]
// @TODO Rename to DocId?
struct InternalId(String);

pub struct ResultItem {
    pub id: Id,
    pub text: Text,
    pub score: f64,
}

pub struct Document {
    pub id: Id,
    pub text: Text,
}

struct IndexData {
    num_of_docs_with_token: usize,
    nums_of_token_occurrences_in_doc: HashMap<InternalId, usize>,
}

pub struct LocalSearch {
    index: Trie<Token, IndexData>,
    next_id: usize,
    document_ids: HashMap<InternalId, Id>, // @TODO merge with `document_texts`? + v
    document_texts: HashMap<InternalId, Text>, // @TODO reference to `InternalId`?  + ^
    total_num_of_tokens_in_text: usize,
    nums_of_tokens_in_text: HashMap<InternalId, usize>, // @TODO merge with `document_texts`?
}

impl LocalSearch {
    pub fn new() -> Self {
        Self {
            index: Trie::new(),
            next_id: 0,
            document_ids: HashMap::new(), // @TODO with capacity and faster hasher or structure ?
            document_texts: HashMap::new(), // @TODO with capacity and faster hasher or structure ?
            total_num_of_tokens_in_text: 0,
            nums_of_tokens_in_text: HashMap::new(), // @TODO with capacity and faster hasher or structure ?
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
        let Document { id, text } = document;
        let internal_id = self.add_document_id(id);

        let mut num_of_tokens = 0;
        for raw_token in tokenize(&text) {
            self.add_token(&process_token(raw_token), &internal_id);
            num_of_tokens += 1;
        }
        self.document_texts.insert(internal_id.clone(), text);
        self.nums_of_tokens_in_text.insert(internal_id, num_of_tokens);
        self.total_num_of_tokens_in_text += num_of_tokens;
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

    // @TODO short strings vs usize?
    fn add_document_id(&mut self, document_id: Id) -> InternalId {
        let internal_id = InternalId(radix_36(self.next_id).to_string());
        self.document_ids.insert(internal_id.clone(), document_id);
        self.next_id += 1;
        internal_id
    }

    fn add_token(&mut self, token: &Token, internal_id: &InternalId) {
        if let Some(index_data) = self.index.get_mut(token) {
            if let Some(num_of_token_occurrences_in_doc) = index_data.nums_of_token_occurrences_in_doc.get_mut(internal_id) {
                *num_of_token_occurrences_in_doc += 1;
            } else {
                index_data.num_of_docs_with_token += 1;
            }
        } else {
            let index_data = IndexData {
                num_of_docs_with_token: 1,
                nums_of_token_occurrences_in_doc: {
                    // @TODO with capacity and faster hasher or structure ?
                    let mut nums_of_token_occurrences_in_doc = HashMap::new();
                    nums_of_token_occurrences_in_doc.insert(internal_id.clone(), 1);
                    nums_of_token_occurrences_in_doc
                }
            };
            self.index.insert(token.clone(), index_data);
        }
    }

    fn add_num_of_tokens_in_text(&mut self, num_of_tokens: usize, internal_id: &InternalId) {

    }
}

fn tokenize(text: &Text) -> impl Iterator<Item=&RawToken> {
    text.unicode_words()
}

fn process_token(raw_token: &RawToken) -> Token {
    raw_token.to_lowercase()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
