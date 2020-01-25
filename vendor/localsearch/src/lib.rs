use indexmap::{IndexMap, indexmap};
use radix_trie::{Trie, TrieCommon};
use unicode_segmentation::UnicodeSegmentation;
use generic_levenshtein;

type DocId = usize;
type Token = String;
type Score = f64;
type TokenCount = usize;
type TokenOccurenceCount = usize;
type DocIdTokenOccurenceCountPairs = IndexMap<DocId, TokenOccurenceCount>;
type Distance = usize;

const EQUAL_DOC_BOOST: f64 = 3.;
const DEFAULT_EDIT_DISTANCE: usize = 1;

pub fn default_tokenizer(text: &str) -> Vec<Token> {
    text
        .unicode_words()
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

pub struct LocalSearch<T> {
    index: Trie<Token, DocIdTokenOccurenceCountPairs>,
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

impl<T> LocalSearch<T> {

    // ------ pub ------

    pub fn new(text_getter: impl (Fn(&T) -> &str) + 'static) -> Self {
        Self::with_capacity(text_getter, 0)
    }

    pub fn with_capacity(text_getter: impl (Fn(&T) -> &str) + 'static, capacity: usize) -> Self {
        Self {
            index: Trie::new(),
            documents: IndexMap::with_capacity(capacity),
            text_getter: Box::new(text_getter),
            tokenizer: Box::new(default_tokenizer),
            doc_id_generator: DocIdGenerator::default(),
            max_edit_distance: DEFAULT_EDIT_DISTANCE,
        }
    }

    pub fn with_documents(documents: Vec<T>, text_getter: impl (Fn(&T) -> &str) + 'static) -> Self {
        let mut local_search = Self::with_capacity(text_getter, documents.len());
        local_search.add_documents(documents);
        local_search
    }

    pub fn set_tokenizer(&mut self, tokenizer: impl for <'a> Fn(&'a str) -> Vec<String> + 'static) {
        self.tokenizer = Box::new(tokenizer);
    }

    pub fn set_max_edit_distance(&mut self, distance: Distance) {
        self.max_edit_distance = distance;
    }

    pub fn add_documents(&mut self, documents: Vec<T>) {
        self.documents.reserve(documents.len());
        for document in documents {
            self.add_document(document)
        }
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<ResultItem<T>> {
        (self.tokenizer)(query)
            .into_iter()
            .flat_map(|token| self.tokens_in_distance(token))
            .flat_map(|(token, distance)| self.search_exact(token, distance))
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

    // ------ private ------

    fn add_document(&mut self, document: T) {
        let doc_id = self.doc_id_generator.next();
        let text = (self.text_getter)(&document);
        let tokens = (self.tokenizer)(text);
        let token_count = tokens.len();

        self.documents.insert(doc_id, (document, token_count));

        for token in tokens {
            self.add_token(token, doc_id);
        }
    }

    fn add_token(&mut self, token: Token, doc_id: DocId) {
        self.index.map_with_default(token, |doc_id_token_occurrence_count_pairs| {
            doc_id_token_occurrence_count_pairs
                .entry(doc_id)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }, indexmap!{ doc_id => 1 })
    }

    // @TODO: Optimize as soon as this is resolved - https://github.com/BurntSushi/fst/issues/60
    fn tokens_in_distance(&self, query_token: Token) -> Vec<(Token, Distance)> {
        self
            .index
            .keys()
            .filter_map(|token| {
                let distance = generic_levenshtein::distance(&query_token, token);
                if distance <= self.max_edit_distance {
                    Some((token.clone(), distance))
                } else {
                    None
                }
            })
            .collect()
    }

    fn search_exact(&self, token: Token, distance: usize) -> IndexMap<DocId, Score> {
        self
            .index
            .get(&token)
            .map(|doc_id_token_occurrence_count_pairs| {
                doc_id_token_occurrence_count_pairs
                    .into_iter()
                    .fold(IndexMap::new(), |mut results, (doc_id, token_occurrence_count)| {
                        let token_count = self.documents.get(doc_id).map(|(_, count)| *count).expect("get token count");
                        let tf_idf = self.tf_idf(*token_occurrence_count, token_count, doc_id_token_occurrence_count_pairs.len());
                        let score = tf_idf * (self.max_edit_distance - distance) as f64;
                        results.insert(*doc_id, score);
                        results
                    })
            })
            .unwrap_or_default()
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
