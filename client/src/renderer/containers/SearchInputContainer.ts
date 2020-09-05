import { connect } from 'react-redux';
import { ThunkDispatch } from 'redux-thunk';

import SearchInput from '../components/SearchInput';
import { RootState } from '../reducers';
import { SearchInputAction, search, addCharacter } from '../actions/searchInputAction';

const mapStateToProps = (state: RootState) => ({
    query: state.searchInput.query,
    searching: state.searchInput.searching,
    results: state.searchInput.results
});

const mapDispatchToProps = (dispatch: ThunkDispatch<any, any, SearchInputAction>) => ({
    search: (query: string) => dispatch(search(query)),
    addCharacter: (input: string) => dispatch(addCharacter(input))
});

export default connect(mapStateToProps, mapDispatchToProps)(SearchInput);
