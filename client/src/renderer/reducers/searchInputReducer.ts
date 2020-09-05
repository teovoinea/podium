import { Reducer, Action } from 'redux';
import { ipcRenderer } from 'electron';
import SearchResult from '../../contracts/SearchResult';

import {
    SearchInputAction,
    AddCharacterAction,
    SEARCH,
    ADDCHARACTER,
    SearchAction,
    IsSearchingAction,
    IS_SEARCHING,
    QuerySuccessfulAction,
    QUERY_SUCCESSFUL,
    DISPLAY_RESULTS,
    DisplayResultsAction
} from '../actions/searchInputAction';

export interface SearchInputState {
    readonly query: string;
    readonly searching: boolean;
    readonly results: SearchResult[];
}

const defaultState: SearchInputState = {
    query: '',
    searching: false,
    results: []
};

export const searchInputReducer: Reducer<SearchInputState> = (
    state = defaultState,
    action: Action
) => {
    switch (action.type) {
        case IS_SEARCHING: {
            // One day we can show a loading indicator that hopefully users will never see
            const searching = action as IsSearchingAction;
            return {
                ...state,
                searching: searching.isSearching
            };
        }
        case DISPLAY_RESULTS: {
            const displayResults = action as DisplayResultsAction;
            if (displayResults.displayingResults) {
                ipcRenderer.send('displayResults');
            } else {
                ipcRenderer.send('shrinkWindow');
            }
            return state;
        }
        case QUERY_SUCCESSFUL: {
            const querySuccessful = action as QuerySuccessfulAction;

            return {
                ...state,
                results: querySuccessful.results
            };
        }
        case ADDCHARACTER: {
            const addAction = action as AddCharacterAction;
            return {
                ...state,
                query: addAction.text
            };
        }
        case SEARCH:
        default: {
            return state;
        }
    }
};
