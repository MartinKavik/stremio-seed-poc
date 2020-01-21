
function index_multisearch(documents) {
    window.my_MiniSearch = new MiniSearch({
        fields: ['name'], // fields to index for full-text search
        storeFields: ['name'] // fields to return with search results
    });
    window.my_MiniSearch.addAll(JSON.parse(documents))
}

function search_multisearch(query) {
//    return window.my_MiniSearch.search(query, { fuzzy: 0.2, /* prefix: true - "the office" - modify weights? */ })
    return window.my_MiniSearch.search(query)
}

console.log("minisearch script finished");

