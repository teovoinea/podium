import * as React from 'react';
import { Icon, InlineIcon } from '@iconify/react';
import bxSearch from '@iconify/icons-bx/bx-search';
import SearchResult from '../../contracts/SearchResult';
import SearchResults from './SearchResults';

require('./SearchInput.scss');

export interface Props {
    query: string;
    searching: boolean;
    results: SearchResult[];

    search: (input: string) => any;
    addCharacter: (input: string) => any;
}

const handleKeyPress = (searchBind: any) => {
    return (e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.charCode === 13) {
            searchBind(e.currentTarget.value);
        }
    };
};

const handleChange = (addCharacterBind: any) => {
    return (e: React.ChangeEvent<HTMLInputElement>) => {
        addCharacterBind(e.target.value);
    };
};

const SearchInput: React.FunctionComponent<Props> = ({ query, results, search, addCharacter }) => (
    <div className="searchRoot">
        <div className="searchInputRoot">
            <Icon icon={bxSearch} id="magnifyingGlass" />
            <input
                className="searchInput"
                type="text"
                value={query}
                onChange={handleChange(addCharacter)}
                onKeyPress={handleKeyPress(search)}
                />
        </div>
        <SearchResults results={results} />
    </div>
);

export default SearchInput;
