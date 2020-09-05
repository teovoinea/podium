import { Action, ActionCreator, AnyAction } from 'redux';
import { ThunkAction, ThunkDispatch } from 'redux-thunk';
import SearchResult from '../../contracts/SearchResult';
import SendSearchQuery from '../../api/client';

export const SEARCH = 'SEARCH';
export const ADDCHARACTER = 'ADD_CHARACTER';
export const IS_SEARCHING = 'IS_SEARCHING';
export const QUERY_SUCCESSFUL = 'QUERY_SUCCESSFUL';
export const DISPLAY_RESULTS = 'DISPLAY_RESULTS';

export interface SearchAction extends Action {
    type: 'SEARCH';
}

export interface AddCharacterAction extends Action {
    type: 'ADD_CHARACTER';
    text: string;
}

export interface IsSearchingAction extends Action {
    type: 'IS_SEARCHING';
    isSearching: boolean;
}

export interface QuerySuccessfulAction extends Action {
    type: 'QUERY_SUCCESSFUL';
    results: SearchResult[];
}

export interface DisplayResultsAction extends Action {
    type: 'DISPLAY_RESULTS';
    displayingResults: boolean;
}

export const search = (query: string): ThunkAction<Promise<void>, {}, {}, AnyAction> => {
    return async (dispatch: ThunkDispatch<{}, {}, AnyAction>): Promise<void> => {
        if (query === null || query.length === 0) {
            dispatch(displayResults(false));
        } else {
            dispatch(isSearching(true));
            const results = await SendSearchQuery(query);
            dispatch(isSearching(false));
            dispatch(querySuccessful(results));
            dispatch(displayResults(true));
        }
    };
};

export const addCharacter: ActionCreator<AddCharacterAction> = (input: string) => ({
    type: ADDCHARACTER,
    text: input
});

export const isSearching: ActionCreator<IsSearchingAction> = (setSearching: boolean) => ({
    type: IS_SEARCHING,
    isSearching: setSearching
});

export const querySuccessful: ActionCreator<QuerySuccessfulAction> = (results: SearchResult[]) => ({
    type: QUERY_SUCCESSFUL,
    results
});

export const displayResults: ActionCreator<DisplayResultsAction> = (
    displayingResults: boolean
) => ({
    type: DISPLAY_RESULTS,
    displayingResults
});

export type SearchInputAction =
    | SearchAction
    | AddCharacterAction
    | IsSearchingAction
    | QuerySuccessfulAction;
