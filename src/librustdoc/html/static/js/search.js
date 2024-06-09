// ignore-tidy-filelength
/* global addClass, getNakedUrl, getSettingValue */
/* global onEachLazy, removeClass, searchState, browserSupportsHistoryApi, exports */

"use strict";


(function() {
    const longItemTypes = [
        "keyword",
        "primitive type",
        "module",
        "extern crate",
        "re-export",
        "struct",
        "enum",
        "function",
        "type alias",
        "static",
        "trait",
        "",
        "trait method",
        "method",
        "struct field",
        "enum variant",
        "macro",
        "assoc type",
        "constant",
        "assoc const",
        "union",
        "foreign type",
        "existential type",
        "attribute macro",
        "derive macro",
        "trait alias",
    ];
    let currentResults;

    // In the search display, allows to switch between tabs.
    function printTab(nb) {
        let iter = 0;
        let foundCurrentTab = false;
        let foundCurrentResultSet = false;
        onEachLazy(document.getElementById("search-tabs").childNodes, elem => {
            if (nb === iter) {
                addClass(elem, "selected");
                foundCurrentTab = true;
            } else {
                removeClass(elem, "selected");
            }
            iter += 1;
        });
        const isTypeSearch = (nb > 0 || iter === 1);
        iter = 0;
        onEachLazy(document.getElementById("results").childNodes, elem => {
            if (nb === iter) {
                addClass(elem, "active");
                foundCurrentResultSet = true;
            } else {
                removeClass(elem, "active");
            }
            iter += 1;
        });
        if (foundCurrentTab && foundCurrentResultSet) {
            searchState.currentTab = nb;
            // Corrections only kick in on type-based searches.
            const correctionsElem = document.getElementsByClassName("search-corrections");
            if (isTypeSearch) {
                removeClass(correctionsElem[0], "hidden");
            } else {
                addClass(correctionsElem[0], "hidden");
            }
        } else if (nb !== 0) {
            printTab(0);
        }
    }

    /**
     * Build an URL with search parameters.
     *
     * @param {string} search            - The current search being performed.
     * @param {string|null} filterCrates - The current filtering crate (if any).
     *
     * @return {string}
     */
    function buildUrl(search, filterCrates) {
        let extra = "?search=" + encodeURIComponent(search);

        if (filterCrates !== null) {
            extra += "&filter-crate=" + encodeURIComponent(filterCrates);
        }
        return getNakedUrl() + extra + window.location.hash;
    }

    /**
     * Return the filtering crate or `null` if there is none.
     *
     * @return {string|null}
     */
    function getFilterCrates() {
        const elem = document.getElementById("crate-search");

        if (elem &&
            elem.value !== "all crates" &&
            window.searchIndex.has(elem.value)
        ) {
            return elem.value;
        }
        return null;
    }

    function nextTab(direction) {
        const next = (searchState.currentTab + direction + 3) % searchState.focusedByTab.length;
        searchState.focusedByTab[searchState.currentTab] = document.activeElement;
        printTab(next);
        focusSearchResult();
    }

    // Focus the first search result on the active tab, or the result that
    // was focused last time this tab was active.
    function focusSearchResult() {
        const target = searchState.focusedByTab[searchState.currentTab] ||
            document.querySelectorAll(".search-results.active a").item(0) ||
            document.querySelectorAll("#search-tabs button").item(searchState.currentTab);
        searchState.focusedByTab[searchState.currentTab] = null;
        if (target) {
            target.focus();
        }
    }

    /**
  * Render a set of search results for a single tab.
  * @param {Array<?>}    array   - The search results for this tab
  * @param {ParsedQuery} query
  * @param {boolean}     display - True if this is the active tab
  */
    async function addTab(array, query, display) {
        const extraClass = display ? " active" : "";

        const output = document.createElement("div");
        if (array.length > 0) {
            output.className = "search-results " + extraClass;

            for (const item of array) {
                const name = item.name;
                const type = itemTypes[item.ty];
                const longType = longItemTypes[item.ty];
                const typeName = longType.length !== 0 ? `${longType}` : "?";

                const link = document.createElement("a");
                link.className = "result-" + type;
                link.href = item.href;

                const resultName = document.createElement("div");
                resultName.className = "result-name";

                resultName.insertAdjacentHTML(
                    "beforeend",
                    `<span class="typename">${typeName}</span>`);
                link.appendChild(resultName);

                let alias = " ";
                if (item.is_alias) {
                    alias = ` <div class="alias">\
<b>${item.alias}</b><i class="grey">&nbsp;- see&nbsp;</i>\
</div>`;
                }
                resultName.insertAdjacentHTML(
                    "beforeend",
                    `<div class="path">${alias}\
${item.displayPath}<span class="${type}">${name}</span>\
</div>`);

                const description = document.createElement("div");
                description.className = "desc";
                description.insertAdjacentHTML("beforeend", item.desc);

                link.appendChild(description);
                output.appendChild(link);
            }
        } else if (query.error === null) {
            output.className = "search-failed" + extraClass;
            output.innerHTML = "No results :(<br/>" +
                "Try on <a href=\"https://duckduckgo.com/?q=" +
                encodeURIComponent("rust " + query.userQuery) +
                "\">DuckDuckGo</a>?<br/><br/>" +
                "Or try looking in one of these:<ul><li>The <a " +
                "href=\"https://doc.rust-lang.org/reference/index.html\">Rust Reference</a> " +
                " for technical details about the language.</li><li><a " +
                "href=\"https://doc.rust-lang.org/rust-by-example/index.html\">Rust By " +
                "Example</a> for expository code examples.</a></li><li>The <a " +
                "href=\"https://doc.rust-lang.org/book/index.html\">Rust Book</a> for " +
                "introductions to language features and the language itself.</li><li><a " +
                "href=\"https://docs.rs\">Docs.rs</a> for documentation of crates released on" +
                " <a href=\"https://crates.io/\">crates.io</a>.</li></ul>";
        }
        return [output, array.length];
    }

    function makeTabHeader(tabNb, text, nbElems) {
        // https://blog.horizon-eda.org/misc/2020/02/19/ui.html
        //
        // CSS runs with `font-variant-numeric: tabular-nums` to ensure all
        // digits are the same width. \u{2007} is a Unicode space character
        // that is defined to be the same width as a digit.
        const fmtNbElems =
            nbElems < 10 ? `\u{2007}(${nbElems})\u{2007}\u{2007}` :
                nbElems < 100 ? `\u{2007}(${nbElems})\u{2007}` :
                    `\u{2007}(${nbElems})`;
        if (searchState.currentTab === tabNb) {
            return "<button class=\"selected\">" + text +
                "<span class=\"count\">" + fmtNbElems + "</span></button>";
        }
        return "<button>" + text + "<span class=\"count\">" + fmtNbElems + "</span></button>";
    }

    /**
     * @param {ResultsTable} results
     * @param {boolean} go_to_first
     * @param {string} filterCrates
     */
    async function showResults(results, go_to_first, filterCrates) {
        const search = searchState.outputElement();
        if (go_to_first || (results.others.length === 1
            && getSettingValue("go-to-only-result") === "true")
        ) {
            // Needed to force re-execution of JS when coming back to a page. Let's take this
            // scenario as example:
            //
            // 1. You have the "Directly go to item in search if there is only one result" option
            //    enabled.
            // 2. You make a search which results only one result, leading you automatically to
            //    this result.
            // 3. You go back to previous page.
            //
            // Now, without the call below, the JS will not be re-executed and the previous state
            // will be used, starting search again since the search input is not empty, leading you
            // back to the previous page again.
            window.onunload = () => { };
            searchState.removeQueryParameters();
            const elem = document.createElement("a");
            elem.href = results.others[0].href;
            removeClass(elem, "active");
            // For firefox, we need the element to be in the DOM so it can be clicked.
            document.body.appendChild(elem);
            elem.click();
            return;
        }
        if (results.query === undefined) {
            results.query = DocSearch.parseQuery(searchState.input.value);
        }

        currentResults = results.query.userQuery;

        const [ret_others, ret_in_args, ret_returned] = await Promise.all([
            addTab(results.others, results.query, true),
            addTab(results.in_args, results.query, false),
            addTab(results.returned, results.query, false),
        ]);

        // Navigate to the relevant tab if the current tab is empty, like in case users search
        // for "-> String". If they had selected another tab previously, they have to click on
        // it again.
        let currentTab = searchState.currentTab;
        if ((currentTab === 0 && ret_others[1] === 0) ||
            (currentTab === 1 && ret_in_args[1] === 0) ||
            (currentTab === 2 && ret_returned[1] === 0)) {
            if (ret_others[1] !== 0) {
                currentTab = 0;
            } else if (ret_in_args[1] !== 0) {
                currentTab = 1;
            } else if (ret_returned[1] !== 0) {
                currentTab = 2;
            }
        }

        let crates = "";
        if (rawSearchIndex.size > 1) {
            crates = " in&nbsp;<div id=\"crate-search-div\"><select id=\"crate-search\">" +
                "<option value=\"all crates\">all crates</option>";
            for (const c of rawSearchIndex.keys()) {
                crates += `<option value="${c}" ${c === filterCrates && "selected"}>${c}</option>`;
            }
            crates += "</select></div>";
        }

        let output = `<h1 class="search-results-title">Results${crates}</h1>`;
        if (results.query.error !== null) {
            const error = results.query.error;
            error.forEach((value, index) => {
                value = value.split("<").join("&lt;").split(">").join("&gt;");
                if (index % 2 !== 0) {
                    error[index] = `<code>${value.replaceAll(" ", "&nbsp;")}</code>`;
                } else {
                    error[index] = value;
                }
            });
            output += `<h3 class="error">Query parser error: "${error.join("")}".</h3>`;
            output += "<div id=\"search-tabs\">" +
                makeTabHeader(0, "In Names", ret_others[1]) +
                "</div>";
            currentTab = 0;
        } else if (results.query.foundElems <= 1 && results.query.returned.length === 0) {
            output += "<div id=\"search-tabs\">" +
                makeTabHeader(0, "In Names", ret_others[1]) +
                makeTabHeader(1, "In Parameters", ret_in_args[1]) +
                makeTabHeader(2, "In Return Types", ret_returned[1]) +
                "</div>";
        } else {
            const signatureTabTitle =
                results.query.elems.length === 0 ? "In Function Return Types" :
                    results.query.returned.length === 0 ? "In Function Parameters" :
                        "In Function Signatures";
            output += "<div id=\"search-tabs\">" +
                makeTabHeader(0, signatureTabTitle, ret_others[1]) +
                "</div>";
            currentTab = 0;
        }

        if (results.query.correction !== null) {
            const orig = results.query.returned.length > 0
                ? results.query.returned[0].name
                : results.query.elems[0].name;
            output += "<h3 class=\"search-corrections\">" +
                `Type "${orig}" not found. ` +
                "Showing results for closest type name " +
                `"${results.query.correction}" instead.</h3>`;
        }
        if (results.query.proposeCorrectionFrom !== null) {
            const orig = results.query.proposeCorrectionFrom;
            const targ = results.query.proposeCorrectionTo;
            output += "<h3 class=\"search-corrections\">" +
                `Type "${orig}" not found and used as generic parameter. ` +
                `Consider searching for "${targ}" instead.</h3>`;
        }

        const resultsElem = document.createElement("div");
        resultsElem.id = "results";
        resultsElem.appendChild(ret_others[0]);
        resultsElem.appendChild(ret_in_args[0]);
        resultsElem.appendChild(ret_returned[0]);

        search.innerHTML = output;
        const crateSearch = document.getElementById("crate-search");
        if (crateSearch) {
            crateSearch.addEventListener("input", updateCrate);
        }
        search.appendChild(resultsElem);
        // Reset focused elements.
        searchState.showResults(search);
        const elems = document.getElementById("search-tabs").childNodes;
        searchState.focusedByTab = [];
        let i = 0;
        for (const elem of elems) {
            const j = i;
            elem.onclick = () => printTab(j);
            searchState.focusedByTab.push(null);
            i += 1;
        }
        printTab(currentTab);
    }

    function updateSearchHistory(url) {
        if (!browserSupportsHistoryApi()) {
            return;
        }
        const params = searchState.getQueryStringParams();
        if (!history.state && !params.search) {
            history.pushState(null, "", url);
        } else {
            history.replaceState(null, "", url);
        }
    }

    /**
     * Perform a search based on the current state of the search input element
     * and display the results.
     * @param {boolean} [forced]
     */
    async function search(forced) {
        const query = DocSearch.parseQuery(searchState.input.value.trim());
        let filterCrates = getFilterCrates();

        if (!forced && query.userQuery === currentResults) {
            if (query.userQuery.length > 0) {
                putBackSearch();
            }
            return;
        }

        searchState.setLoadingSearch();

        const params = searchState.getQueryStringParams();

        // In case we have no information about the saved crate and there is a URL query parameter,
        // we override it with the URL query parameter.
        if (filterCrates === null && params["filter-crate"] !== undefined) {
            filterCrates = params["filter-crate"];
        }

        // Update document title to maintain a meaningful browser history
        searchState.title = "Results for " + query.original + " - Rust";

        // Because searching is incremental by character, only the most
        // recent search query is added to the browser history.
        updateSearchHistory(buildUrl(query.original, filterCrates));

        await showResults(
            await docSearch.execQuery(query, filterCrates, window.currentCrate),
            params.go_to_first,
            filterCrates);
    }

    /**
         * Callback for when the search form is submitted.
         * @param {Event} [e] - The event that triggered this call, if any
         */
    function onSearchSubmit(e) {
        e.preventDefault();
        searchState.clearInputTimeout();
        search();
    }

    function putBackSearch() {
        const search_input = searchState.input;
        if (!searchState.input) {
            return;
        }
        if (search_input.value !== "" && !searchState.isDisplayed()) {
            searchState.showResults();
            if (browserSupportsHistoryApi()) {
                history.replaceState(null, "",
                    buildUrl(search_input.value, getFilterCrates()));
            }
            document.title = searchState.title;
        }
    }

    function registerSearchEvents() {
        const params = searchState.getQueryStringParams();

        // Populate search bar with query string search term when provided,
        // but only if the input bar is empty. This avoid the obnoxious issue
        // where you start trying to do a search, and the index loads, and
        // suddenly your search is gone!
        if (searchState.input.value === "") {
            searchState.input.value = params.search || "";
        }

        const searchAfter500ms = () => {
            searchState.clearInputTimeout();
            if (searchState.input.value.length === 0) {
                searchState.hideResults();
            } else {
                searchState.timeout = setTimeout(search, 500);
            }
        };
        searchState.input.onkeyup = searchAfter500ms;
        searchState.input.oninput = searchAfter500ms;
        document.getElementsByClassName("search-form")[0].onsubmit = onSearchSubmit;
        searchState.input.onchange = e => {
            if (e.target !== document.activeElement) {
                // To prevent doing anything when it's from a blur event.
                return;
            }
            // Do NOT e.preventDefault() here. It will prevent pasting.
            searchState.clearInputTimeout();
            // zero-timeout necessary here because at the time of event handler execution the
            // pasted content is not in the input field yet. Shouldn’t make any difference for
            // change, though.
            setTimeout(search, 0);
        };
        searchState.input.onpaste = searchState.input.onchange;

        searchState.outputElement().addEventListener("keydown", e => {
            // We only handle unmodified keystrokes here. We don't want to interfere with,
            // for instance, alt-left and alt-right for history navigation.
            if (e.altKey || e.ctrlKey || e.shiftKey || e.metaKey) {
                return;
            }
            // up and down arrow select next/previous search result, or the
            // search box if we're already at the top.
            if (e.which === 38) { // up
                const previous = document.activeElement.previousElementSibling;
                if (previous) {
                    previous.focus();
                } else {
                    searchState.focus();
                }
                e.preventDefault();
            } else if (e.which === 40) { // down
                const next = document.activeElement.nextElementSibling;
                if (next) {
                    next.focus();
                }
                const rect = document.activeElement.getBoundingClientRect();
                if (window.innerHeight - rect.bottom < rect.height) {
                    window.scrollBy(0, rect.height);
                }
                e.preventDefault();
            } else if (e.which === 37) { // left
                nextTab(-1);
                e.preventDefault();
            } else if (e.which === 39) { // right
                nextTab(1);
                e.preventDefault();
            }
        });

        searchState.input.addEventListener("keydown", e => {
            if (e.which === 40) { // down
                focusSearchResult();
                e.preventDefault();
            }
        });

        searchState.input.addEventListener("focus", () => {
            putBackSearch();
        });

        searchState.input.addEventListener("blur", () => {
            searchState.input.placeholder = searchState.input.origPlaceholder;
        });

        // Push and pop states are used to add search results to the browser
        // history.
        if (browserSupportsHistoryApi()) {
            // Store the previous <title> so we can revert back to it later.
            const previousTitle = document.title;

            window.addEventListener("popstate", e => {
                const params = searchState.getQueryStringParams();
                // Revert to the previous title manually since the History
                // API ignores the title parameter.
                document.title = previousTitle;
                // When browsing forward to search results the previous
                // search will be repeated, so the currentResults are
                // cleared to ensure the search is successful.
                currentResults = null;
                // Synchronize search bar with query string state and
                // perform the search. This will empty the bar if there's
                // nothing there, which lets you really go back to a
                // previous state with nothing in the bar.
                if (params.search && params.search.length > 0) {
                    searchState.input.value = params.search;
                    // Some browsers fire "onpopstate" for every page load
                    // (Chrome), while others fire the event only when actually
                    // popping a state (Firefox), which is why search() is
                    // called both here and at the end of the startSearch()
                    // function.
                    e.preventDefault();
                    search();
                } else {
                    searchState.input.value = "";
                    // When browsing back from search results the main page
                    // visibility must be reset.
                    searchState.hideResults();
                }
            });
        }

        // This is required in firefox to avoid this problem: Navigating to a search result
        // with the keyboard, hitting enter, and then hitting back would take you back to
        // the doc page, rather than the search that should overlay it.
        // This was an interaction between the back-forward cache and our handlers
        // that try to sync state between the URL and the search input. To work around it,
        // do a small amount of re-init on page show.
        window.onpageshow = () => {
            const qSearch = searchState.getQueryStringParams().search;
            if (searchState.input.value === "" && qSearch) {
                searchState.input.value = qSearch;
            }
            search();
        };
    }

    function updateCrate(ev) {
        if (ev.target.value === "all crates") {
            // If we don't remove it from the URL, it'll be picked up again by the search.
            const query = searchState.input.value.trim();
            updateSearchHistory(buildUrl(query, null));
        }
        // In case you "cut" the entry from the search input, then change the crate filter
        // before paste back the previous search, you get the old search results without
        // the filter. To prevent this, we need to remove the previous results.
        currentResults = null;
        search(true);
    }

    if (typeof window !== "undefined") {
        registerSearchEvents();
        if (window.searchState.getQueryStringParams().search) {
            search();
        }
    }

    function initSearch(rawSearchIndex) {
        if (typeof window !== "undefined") {
            window.docSearch = new window.DocSearch(rawSearchIndex);
        } else if (typeof exports !== "undefined") {
            exports.docSearch = new exports.DocSearch(rawSearchIndex);
        }
    }

    if (typeof window !== "undefined") {
        window.initSearch = initSearch;
        if (window.searchIndex !== undefined) {
            initSearch(window.searchIndex);
        }
    } else {
        // Running in Node, not a browser. Run initSearch just to produce the
        // exports.
        initSearch(new Map());
    }


})();
