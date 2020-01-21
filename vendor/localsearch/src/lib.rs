use std::collections::HashMap;
use radix_trie::Trie;
use radix_fmt::radix_36;
use unicode_segmentation::UnicodeSegmentation;

// @TODO new types?
pub type Id = String;
pub type Text = String;
type RawToken = str;
type Token = String;
type Score = f64;

#[derive(PartialEq, Eq, Hash, Clone)]
// @TODO Rename to DocId?
#[derive(Debug)]
pub struct InternalId(String);

pub struct ResultItem {
    pub id: Id,
    pub text: Text,
    pub score: f64,
}

pub struct Document {
    pub id: Id,
    pub text: Text,
}

// @TODO remove pub
#[derive(Debug)]
pub struct IndexData {
    // @TODO rename to document_frequency?  // @TODO can get from the next field
    num_of_texts_with_token: usize,
    // @TODO rename to term_frequency or token_frequency?
    pub nums_of_token_occurrences_in_text: HashMap<InternalId, usize>,
}

struct SearchQuery {
    token: Token,
    fuzzy: bool,
    prefix: bool,
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
    // @TODO remove
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }

    // @TODO remove
    pub fn total_length(&self) -> usize {
        self.total_num_of_tokens_in_text
    }

    // @TODO remove
    pub fn average_length(&self) -> f64 {
        self.total_num_of_tokens_in_text as f64 / self.document_ids.len() as f64
    }

    // @TODO remove
    pub fn token_the(&self) -> &IndexData {
        self.index.get("the").unwrap()
    }

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
        let results: Vec<HashMap<InternalId, Score>> =
            tokenize_query(query)
                .map(process_query_token)
                .map(token_to_search_query)
                .map(|search_query| self.execute_search_query(search_query))
                .collect();

        // @TODO combinators, optimize, refactor..
        results.
            into_iter()
            .map(|hash_map: HashMap<InternalId, Score> | {
                let mut vec: Vec<(InternalId, Score)> = hash_map.into_iter().collect();
                vec.sort_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).expect("compare scores"));
                vec
            })
            .map(|vec| {
                vec.into_iter().map(|(internal_id, score)| {
                    ResultItem {
                        score,
                        id: self.document_ids.get(&internal_id).expect("get document id").clone(),
                        text: self.document_texts.get(&internal_id).expect("get document id").clone(),
                    }
                })
            })
            .flatten()
            .take(max_results)
            .collect()
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
            if let Some(num_of_token_occurrences_in_text) = index_data.nums_of_token_occurrences_in_text.get_mut(internal_id) {
                *num_of_token_occurrences_in_text += 1;
            } else {
                index_data.num_of_texts_with_token += 1;
                index_data.nums_of_token_occurrences_in_text.insert(internal_id.clone(), 1);
            }
        } else {
            let index_data = IndexData {
                num_of_texts_with_token: 1,
                nums_of_token_occurrences_in_text: {
                    // @TODO with capacity and faster hasher or structure ?
                    let mut nums_of_token_occurrences_in_text = HashMap::new();
                    nums_of_token_occurrences_in_text.insert(internal_id.clone(), 1);
                    nums_of_token_occurrences_in_text
                }
            };
            self.index.insert(token.clone(), index_data);
        }
    }

    fn execute_search_query(&self, search_query: SearchQuery) -> HashMap<InternalId, Score> {
        let results = self.execute_exact_search(&search_query.token);
        results
    }

    fn execute_exact_search(&self, token: &Token) -> HashMap<InternalId, Score> {
        if let Some(index_data) = self.index.get(token) {
            let mut results = HashMap::new();
            let average_num_of_tokens_in_text = self.total_num_of_tokens_in_text as f64 / self.document_ids.len() as f64;

            for (internal_id, num_of_token_occurrences) in &index_data.nums_of_token_occurrences_in_text {
                let normalized_num_of_token_occurrences = *num_of_token_occurrences as f64 / average_num_of_tokens_in_text;
                let score = self.score(index_data.num_of_texts_with_token, *num_of_token_occurrences, normalized_num_of_token_occurrences);
                results.insert(internal_id.clone(), score);
            }
            results
        } else {
            HashMap::new() // @TODO with capacity and faster hasher or structure ?
        }
    }

    fn score(&self, num_of_texts_with_token: usize, num_of_token_occurrences: usize, normalized_num_of_token_occurrences: f64) -> Score {
        self.tf_idf(num_of_texts_with_token, num_of_token_occurrences) / normalized_num_of_token_occurrences
    }

    fn tf_idf(&self, tf: usize, df: usize) -> f64 {
        let n = self.document_ids.len();
        tf as f64 * f64::ln(n as f64 / df as f64)
    }
}

fn tokenize(text: &str) -> impl Iterator<Item=&RawToken> {
    text.unicode_words()
}

fn tokenize_query(query: &str) -> impl Iterator<Item=&RawToken> {
    tokenize(query)
}

fn process_token(raw_token: &RawToken) -> Token {
    raw_token.to_lowercase()
}

fn process_query_token(raw_token: &RawToken) -> Token {
    process_token(raw_token)
}

fn token_to_search_query(token: Token) -> SearchQuery {
    SearchQuery {
        token,
        prefix: false, // @TODO not implemented
        fuzzy: false, // @TODO not implemented
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
