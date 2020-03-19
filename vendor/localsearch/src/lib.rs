use fxhash::{FxHashMap, FxBuildHasher};
use fst::{self, IntoStreamer, Automaton, Streamer};
use std::iter::FromIterator;

pub type Token = String;
pub type Distance = usize;
pub type Score = f64;
type DocId = usize;
type TokenCount = usize;
type TokenOccurenceCount = usize;

mod levenshtein;

// ------ Defaults ------

const DEFAULT_MAX_EDIT_DISTANCE: usize = 1;
const DEFAULT_PREFIX_BOOST: f64 = 1.5;

pub fn default_tokenizer(text: &str) -> Vec<Token> {
    text
        .split_whitespace()
        .map(str::to_lowercase)
        .collect()
}

// ------ LocalSearchBuilder ------

pub struct LocalSearchBuilder<T> {
    documents: Vec<T>,
    text_extractor: Box<dyn Fn(&T) -> &str>,
    tokenizer: Option<Box<dyn Fn(&str) -> Vec<Token>>>,
    max_edit_distance: Option<Distance>,
    prefix_boost: Option<f64>,
}

impl<T> LocalSearchBuilder<T> {
    pub fn new(documents: Vec<T>, text_extractor: impl Fn(&T) -> &str + 'static) -> Self {
        Self {
            documents,
            text_extractor: Box::new(text_extractor),
            tokenizer: None,
            max_edit_distance: None,
            prefix_boost: None,
        }
    }

    pub fn tokenizer(mut self, tokenizer: impl Fn(&str) -> Vec<Token> + 'static) -> Self {
        self.tokenizer = Some(Box::new(tokenizer));
        self
    }

    pub fn max_edit_distance(mut self, max_edit_distance: Distance) -> Self {
        self.max_edit_distance = Some(max_edit_distance);
        self
    }

    pub fn prefix_boost(mut self, prefix_boost: f64) -> Self {
        self.prefix_boost = Some(prefix_boost);
        self
    }

    pub fn build(self) -> LocalSearch<T> {
        let mut documents = FxHashMap::<DocId, (T, TokenCount)>::with_capacity_and_hasher(
            self.documents.len(), FxBuildHasher::default()
        );
        let tokenizer = self.tokenizer.unwrap_or_else(|| Box::new(default_tokenizer));

        let mut token_and_pairs_map = FxHashMap::<Token, FxHashMap<DocId, TokenOccurenceCount>>::with_capacity_and_hasher(
            self.documents.len(), FxBuildHasher::default()
        );

        for (doc_id, document) in self.documents.into_iter().enumerate() {
            let text = (self.text_extractor)(&document);
            let tokens = tokenizer(text);
            let token_count = tokens.len();
            documents.insert(doc_id, (document, token_count));

            for token in tokens {
                token_and_pairs_map
                    .entry(token)
                    .and_modify(|doc_id_token_occurrence_count_pairs| {
                        doc_id_token_occurrence_count_pairs
                            .entry(doc_id)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    })
                    .or_insert(FxHashMap::from_iter(vec![(doc_id, 1)]));
            }
        }

        let mut token_and_pairs_vec = token_and_pairs_map.into_iter().collect::<Vec<_>>();
        token_and_pairs_vec.sort_unstable_by(|(token_a, _), (token_b, _)| {
            token_a.cmp(token_b)
        });
        let (tokens, pairs): (Vec<_>, Vec<_>) = token_and_pairs_vec.into_iter().unzip();

        let index = Index {
            token_and_pair_index_map: fst::Map::from_iter(tokens.into_iter().zip(0..))
                .expect("build fst map from given documents"),
            pairs
        };

        LocalSearch {
            documents,
            tokenizer,
            max_edit_distance: self.max_edit_distance.unwrap_or(DEFAULT_MAX_EDIT_DISTANCE),
            prefix_boost: self.prefix_boost.unwrap_or(DEFAULT_PREFIX_BOOST),
            index,
        }
    }
}

// ------ Index ------

pub struct Index {
    token_and_pair_index_map: fst::Map<Vec<u8>>,
    pairs: Vec<FxHashMap<DocId, TokenOccurenceCount>>,
}

// ------ RelatedTokenData ------

#[derive(Clone, Copy)]
struct RelatedTokenData {
    distance: Option<Distance>,
    same_prefix: bool,
}

// ------ LocalSearch ------

pub struct LocalSearch<T> {
    documents: FxHashMap<DocId, (T, TokenCount)>,
    tokenizer: Box<dyn Fn(&str) -> Vec<Token>>,
    max_edit_distance: Distance,
    prefix_boost: f64,
    index: Index,
}

impl<T> LocalSearch<T> {

    // ------ pub ------

    pub fn builder(documents: Vec<T>, text_extractor: impl Fn(&T) -> &str + 'static) -> LocalSearchBuilder<T> {
        LocalSearchBuilder::new(documents, text_extractor)
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<(&T, Score)> {
        let mut doc_ids_and_scores =
            (self.tokenizer)(query)
                .into_iter()
                .flat_map(|token| self.related_tokens(&token))
                .flat_map(|(token, data)| self.search_exact(token, data))
                .fold(FxHashMap::default(), |mut results, (doc_id, score)| {
                    results
                        .entry(doc_id)
                        .and_modify(|merged_score| *merged_score = *merged_score + score)
                        .or_insert(score);
                    results
                })
                .into_iter()
                .collect::<Vec<_>>();
        doc_ids_and_scores.sort_unstable_by(|(_, score_a), (_, score_b)| {
            score_b.partial_cmp(score_a).expect("sort scores")
        });
        doc_ids_and_scores.truncate(max_results);
        doc_ids_and_scores
            .into_iter()
            .map(|(doc_id, score)|
                (
                    self.documents.get(&doc_id).map(|(doc, _)| doc).expect("get document"),
                    score
                )
            )
            .collect()
    }

    pub fn autocomplete(&self, query_token: &str, max_results: usize) -> Vec<String> {
        let mut token_stream =
            self
                .index
                .token_and_pair_index_map
                .search(fst::automaton::Str::new(query_token).starts_with())
                .into_stream();

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

    fn related_tokens(&self, query_token: &str) -> FxHashMap<Token, RelatedTokenData> {
        let mut related_tokens = FxHashMap::default();
        self.add_tokens_in_distance(query_token, &mut related_tokens);
        self.add_tokens_with_prefix(query_token, &mut related_tokens);
        related_tokens
    }

    fn add_tokens_in_distance(&self, query_token: &str, related_tokens: &mut FxHashMap<Token, RelatedTokenData>) {
        let lev_query = levenshtein::Levenshtein::new(query_token, self.max_edit_distance)
            .expect("create Levenshtein automaton");

        let mut token_stream =
            self
                .index
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

    fn add_tokens_with_prefix(&self, query_token: &str, related_tokens: &mut FxHashMap<Token, RelatedTokenData>) {
        let mut token_stream =
            self
                .index
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

    fn search_exact(&self, token: Token, token_data: RelatedTokenData) -> FxHashMap<DocId, Score> {
        self.index.token_and_pair_index_map
            .get(&token)
            .map(|pair_index| {
                let doc_id_token_occurrence_count_pairs =
                    self.index.pairs.get(pair_index as usize).expect("get pairs");

                doc_id_token_occurrence_count_pairs
                    .into_iter()
                    .fold(FxHashMap::default(), |mut results, (doc_id, token_occurrence_count)| {
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
            self.prefix_boost
        } else {
            1.
        };
        (1. + tf_idf) * distance_boost * prefix_boost
    }

    // https://towardsdatascience.com/tf-term-frequency-idf-inverse-document-frequency-from-scratch-in-python-6c2b61b78558
    fn tf_idf(&self, token_occurrence_count: usize, token_count: usize, num_of_docs_with_token: usize) -> f64 {
        // Term Frequency (TF)
        // tf(t,d) = count of t in d / number of words in d
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
