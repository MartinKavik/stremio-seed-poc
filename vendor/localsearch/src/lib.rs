use indexmap::{IndexMap, indexmap};
use fst::{self, IntoStreamer, Automaton, Streamer};
use std::{collections::BTreeMap, mem};
use seed::log;

type DocId = usize;
type Token = String;
type Score = f64;
type TokenCount = usize;
type TokenOccurenceCount = usize;
type DocIdTokenOccurenceCountPairs = IndexMap<DocId, TokenOccurenceCount>;
type Distance = usize;

mod levenshtein;

const DEFAULT_EDIT_DISTANCE: usize = 1;
const EQUAL_DOC_BOOST: f64 = 3.;
const PREFIX_BOOST: f64 = 1.5;

pub fn default_tokenizer(text: &str) -> Vec<Token> {
    text
        .split_whitespace()
        .map(str::to_lowercase)
        .collect()
}

pub struct ResultItem<'a, T> {
    pub document: &'a T,
    pub score: f64,
}

impl<T: Clone> ResultItem<'_, T> {
    pub fn to_owned_result(&self) -> ResultItemOwned<T> {
        ResultItemOwned {
            document: self.document.clone(),
            score: self.score,
        }
    }
}

pub struct ResultItemOwned<T> {
    pub document: T,
    pub score: f64,
}

#[derive(Default)]
pub struct Store {
    token_and_pair_index_map: fst::Map<Vec<u8>>,
    pairs: Vec<DocIdTokenOccurenceCountPairs>,
}

// impl Store {
//     fn new() -> Self {
//         Self {
//             tokens_and_pair_indices: Map::default(),
//             pairs: Vec::default()
//         }
//     }
// }

pub struct LocalSearch<T> {
    store: Store,
    documents: IndexMap<DocId, (T, TokenCount)>,
    text_getter: Box<dyn Fn(&T) -> &str>,
    tokenizer: Box<dyn Fn(&str) -> Vec<Token>>,
    doc_id_generator: DocIdGenerator,
    max_edit_distance: Distance,
}

#[derive(Default)]
struct DocIdGenerator(DocId);

impl DocIdGenerator {
    pub fn next(&mut self) -> DocId {
        self.0 += 1;
        self.0
    }
}

#[derive(Clone, Copy)]
struct RelatedTokenData {
    distance: Option<Distance>,
    same_prefix: bool,
}

impl<T> LocalSearch<T> {

    // ------ pub ------

    pub fn new(text_getter: impl (Fn(&T) -> &str) + 'static) -> Self {
        Self {
            store: Store::default(),
            documents: IndexMap::new(),
            text_getter: Box::new(text_getter),
            tokenizer: Box::new(default_tokenizer),
            doc_id_generator: DocIdGenerator::default(),
            max_edit_distance: DEFAULT_EDIT_DISTANCE,
        }
    }

    pub fn set_tokenizer(&mut self, tokenizer: impl for <'a> Fn(&'a str) -> Vec<String> + 'static) {
        self.tokenizer = Box::new(tokenizer);
    }

    pub fn set_max_edit_distance(&mut self, distance: Distance) {
        self.max_edit_distance = distance;
    }

    pub fn set_documents(&mut self, documents: Vec<T>) {
        self.documents = IndexMap::with_capacity(documents.len());
        self.doc_id_generator = DocIdGenerator::default();

        let mut token_and_pairs_map = BTreeMap::<Token, DocIdTokenOccurenceCountPairs>::new();

        for document in documents {
            let doc_id = self.doc_id_generator.next();
            let text = (self.text_getter)(&document);
            let tokens = (self.tokenizer)(text);
            let token_count = tokens.len();
            self.documents.insert(doc_id, (document, token_count));

            for token in tokens {
                token_and_pairs_map
                    .entry(token)
                    .and_modify(|doc_id_token_occurrence_count_pairs| {
                        doc_id_token_occurrence_count_pairs
                            .entry(doc_id)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    })
                    .or_insert(indexmap! {doc_id => 1});
            }
        }

        self.store.token_and_pair_index_map = fst::Map::from_iter(
            token_and_pairs_map
                    .keys()
                    .zip(0..)
        ).expect("build fst map from given documents");

        self.store.pairs =
            token_and_pairs_map
                .values_mut()
                .map(|pairs| mem::take(pairs))
                .collect();
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<ResultItem<T>> {
        (self.tokenizer)(query)
            .into_iter()
            .flat_map(|token| self.related_tokens(&token))
            .flat_map(|(token, data)| self.search_exact(token, data))
            .fold(IndexMap::new(), |mut results, (doc_id, score)| {
                results
                    .entry(doc_id)
                    .and_modify(|merged_score| *merged_score = (*merged_score + score) * EQUAL_DOC_BOOST)
                    .or_insert(score);
                results
            })
            .sorted_by(|_, score_a, _, score_b| score_b.partial_cmp(score_a).expect("compare score"))
            .take(max_results)
            .map(|(doc_id, score)| {
                ResultItem {
                    document: self.documents.get(&doc_id).map(|(doc, _)| doc).expect("get document"),
                    score,
                }
            })
            .collect()
    }

    pub fn autocomplete(&self, query_token: &str, max_results: usize) -> Vec<String> {
        let mut token_stream =
            self
                .store
                .token_and_pair_index_map
                .search(fst::automaton::Str::new(query_token).starts_with())
                .into_stream();

        // Note: fst streams don't support combinators like `take`.
        // Otherwise the code below can be refactored to `.take(max_results).into_str_keys()
        let mut tokens = Vec::new();
        while let Some((token, _)) = token_stream.next() {
            let token = String::from_utf8(token.to_vec())
                .expect("cannot convert token to valid UTF-8 String");
            tokens.push(token);
            if tokens.len() == max_results { break }
        }
        tokens
    }

    // ------ private ------

    fn related_tokens(&self, query_token: &str) -> IndexMap<Token, RelatedTokenData> {
        let mut related_tokens = IndexMap::new();
        self.add_tokens_in_distance(query_token, &mut related_tokens);
        self.add_tokens_with_prefix(query_token, &mut related_tokens);
        related_tokens
    }

    fn add_tokens_in_distance(&self, query_token: &str, related_tokens: &mut IndexMap<Token, RelatedTokenData>) {
        let lev_query = levenshtein::Levenshtein::new(query_token, self.max_edit_distance)
            .expect("create Levenshtein automaton");

        let mut token_stream =
            self
                .store
                .token_and_pair_index_map
                .search_with_state(lev_query)
                .into_stream();

        while let Some((token, _, Some(levenshtein::AutomatonState { distance, .. }))) = token_stream.next() {
            let token = String::from_utf8(token.to_vec())
                .expect("cannot convert token to valid UTF-8 String");

            related_tokens
                .entry(token)
                .and_modify(|related_token_data| related_token_data.distance = distance)
                .or_insert(RelatedTokenData {
                    distance,
                    same_prefix: false,
                });
        }
    }

    fn add_tokens_with_prefix(&self, query_token: &str, related_tokens: &mut IndexMap<Token, RelatedTokenData>) {
        let mut token_stream =
            self
                .store
                .token_and_pair_index_map
                .search(fst::automaton::Str::new(query_token).starts_with())
                .into_stream();

        while let Some((token, _)) = token_stream.next() {
            let token = String::from_utf8(token.to_vec())
                .expect("cannot convert token to valid UTF-8 String");

            related_tokens
                .entry(token)
                .and_modify(|related_token_data| related_token_data.same_prefix = true)
                .or_insert(RelatedTokenData {
                    distance: None,
                    same_prefix: true,
                });
        }
    }

    fn search_exact(&self, token: Token, token_data: RelatedTokenData) -> IndexMap<DocId, Score> {
        self.store.token_and_pair_index_map
            .get(&token)
            .map(|pair_index| {
                let doc_id_token_occurrence_count_pairs =
                    self.store.pairs.get(pair_index as usize).expect("get pairs");

                doc_id_token_occurrence_count_pairs
                    .into_iter()
                    .fold(IndexMap::new(), |mut results, (doc_id, token_occurrence_count)| {
                        let token_count = self.documents.get(doc_id).map(|(_, count)| *count).expect("get token count");
                        let tf_idf = self.tf_idf(*token_occurrence_count, token_count, doc_id_token_occurrence_count_pairs.len());
                        let score = self.score(tf_idf, token_data);
                        results.insert(*doc_id, score);
                        results
                    })
            })
            .unwrap_or_default()
    }

    fn score(&self, tf_idf: f64, token_data: RelatedTokenData) -> Score {
        let distance_boost = token_data.distance.map(|distance| self.max_edit_distance + 1 - distance).unwrap_or(1) as f64;
        let prefix_boost = if token_data.same_prefix {
            PREFIX_BOOST
        } else {
            1.
        };
        (1. + tf_idf) * distance_boost * prefix_boost
    }

    // https://towardsdatascience.com/tf-term-frequency-idf-inverse-document-frequency-from-scratch-in-python-6c2b61b78558
    fn tf_idf(&self, token_occurrence_count: usize, token_count: usize, num_of_docs_with_token: usize) -> f64 {
        // Term Frequency (TF)
        // tf(t,d) = count of t in d / number of words in d
//        let tf = token_occurrence_count as f64 / token_count as f64;
        let tf = token_occurrence_count as f64 / token_count as f64;

        // Document Frequency (DF)
        // df(t) = occurrence of t in documents
        let df = num_of_docs_with_token as f64;

        // Inverse Document Frequency (IDF)
        // idf(t) = log(N/(df + 1))
        let n = self.documents.len() as f64;
        let idf = (n / (df + 1.0)).log10();

        // tf-idf(t, d) = tf(t, d) * log(N/(df + 1))
        tf * idf
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
