import { combineReducers } from 'redux';

import { CounterState, counterReducer } from './counterReducer';
import { SearchInputState, searchInputReducer } from './searchInputReducer';

export interface RootState {
    counter: CounterState;
    searchInput: SearchInputState;
}

export const rootReducer = combineReducers<RootState | undefined>({
    counter: counterReducer,
    searchInput: searchInputReducer
});
