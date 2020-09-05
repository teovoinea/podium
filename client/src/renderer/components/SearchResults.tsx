import * as React from 'react';
import SearchResult from '../../contracts/SearchResult';
import SearchResultsRow from './SearchResultsRow';

export interface Props {
    results: SearchResult[];
}

const SearchResults: React.FunctionComponent<Props> = ({ results }) => (
    <div className="searchResultsRoot">
        {/* Eventually, this will also display the preview pane and the headings in the list */}
        {results.map(result => (
            <SearchResultsRow result={result} key={result.location} />
        ))}
    </div>
);

export default SearchResults;
