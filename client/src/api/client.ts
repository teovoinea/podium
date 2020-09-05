import axios from 'axios';
import PodiumSearchResult from '../contracts/PodiumSearchResult';
import SearchResult from '../contracts/SearchResult';

const SendSearchQuery = async (queryString: string) => {
    const result = await axios(`http://127.0.0.1:8080/search/${queryString}`);
    const podResults = result.data as PodiumSearchResult[];
    const searchResults: SearchResult[] = [];
    podResults.forEach(podResult => {
        podResult.location.forEach(l => {
            searchResults.push({
                title: podResult.title,
                location: l,
                body: podResult.body
            } as SearchResult);
        });
    });
    return searchResults;
};

export default SendSearchQuery;
