import * as React from 'react';
import { Icon, InlineIcon } from '@iconify/react';
import typescriptIcon from '@iconify/icons-logos/typescript-icon';
import { shell } from 'electron';
import SearchResult from '../../contracts/SearchResult';

export interface Props {
    result: SearchResult;
}

require('./SearchResultsRow.scss');

const openSelectedFile = (result: SearchResult) => {
    shell.openItem(result.location);
};

const SearchResultsRow: React.FunctionComponent<Props> = ({ result }) => (
    <div className="searchResultsRow" onClick={() => openSelectedFile(result)}>
        <div className="searchResultsRowContent">
            <Icon icon={typescriptIcon} className="searchResultsRowIcon" />
            <span className="searchResultsRowText">{result.location}</span>
        </div>
    </div>
);

export default SearchResultsRow;
