import { hot } from 'react-hot-loader/root';
import * as React from 'react';

import SearchInputContainer from '../containers/SearchInputContainer';

const Application = () => (
    <div>
        <SearchInputContainer />
    </div>
);

export default hot(Application);
