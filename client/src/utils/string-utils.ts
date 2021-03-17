const isNullOrWhitespace = (str: string): boolean => {
    return str === null || str.match(/^ *$/) !== null;
}

export default isNullOrWhitespace;

